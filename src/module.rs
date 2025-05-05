use heck::ToUpperCamelCase;
use serde::{Deserialize, Serialize};

use crate::generated::{ActionId, GameValueId};

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    #[serde(deserialize_with = "deserialize_handlers")]
    pub handlers: Vec<Line>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    #[serde(rename = "type")]
    pub line_type: LineType,
    pub position: u8,
    pub operations: Vec<Op>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum LineType {
    Process,
    Function,
    Event,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Op {
    pub action: ActionId,
    pub values: Vec<NamedValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NamedValue {
    name: String,
    #[serde(deserialize_with = "deserialize_value")]
    value: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Value {
    Array {
        values: Vec<Value>,
    },
    Block {
        block: String,
    },
    Enum {
        #[serde(rename = "enum", deserialize_with = "deserialize_upper_camel")]
        value: String,
    },
    Item {
        item: String,
    },
    Location {
        x: f64,
        y: f64,
        z: f64,
        yaw: f64,
        pitch: f64,
    },
    Number {
        number: Number,
    },
    Particle,
    Potion {
        potion: String,
        amplifier: i16,
        duration: i16,
    },
    Sound {
        sound: String,
        pitch: f32,
        volume: f32,
        variaton: String,
        source: String,
    },
    Text {
        text: String,
        parsing: TextParsing,
    },
    Variable {
        variable: String,
        scope: VariableScope,
    },
    Vector {
        x: f64,
        y: f64,
        z: f64,
    },
    GameValue {
        game_value: GameValueId,
        selection: String,
    },

    Error,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[repr(u8)]
pub enum Number {
    Simple(f64),
    Calc(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum TextParsing {
    Legacy,
    Plain,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum VariableScope {
    Local,
    Global,
    Save,
}

fn deserialize_value<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Value::deserialize(deserializer).unwrap_or(Value::Error))
}

fn deserialize_upper_camel<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.to_upper_camel_case())
}

fn deserialize_handlers<'de, D>(deserializer: D) -> Result<Vec<Line>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Vec<serde_json::Value> = Deserialize::deserialize(deserializer)?;
    let mut lines = Vec::new();

    for value in v {
        if let Some(line) = value.as_object() {
            let line: Line = serde_json::from_value(serde_json::Value::Object(line.clone()))
                .map_err(serde::de::Error::custom)?;
            lines.push(line);
        }
    }

    Ok(lines)
}
