use heck::ToUpperCamelCase;
use serde::{Deserialize, Serialize};
use serde_json::Number;

use crate::generated::ActionId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    pub handlers: Vec<Line>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    #[serde(rename = "type")]
    pub line_type: LineType,
    pub position: u32,
    pub operations: Vec<Op>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineType {
    Function,
    Event,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Op {
    pub action: ActionId,
    #[serde(rename = "values")]
    pub args: Vec<Arga>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Arga {
    name: String,
    #[serde(deserialize_with = "deserialize_arg")]
    value: Arg,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Arg {
    Any,
    Array,
    Block,
    Enum {
        #[serde(rename = "enum", deserialize_with = "deserialize_upper_camel")]
        value: String,
    },
    Item,
    Location,
    Map,
    Number {
        number: Number,
    },
    Particle,
    Potion,
    Sound,
    Text,
    Variable {
        variable: String,
        scope: VariableScope,
    },
    Vector,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableScope {
    Local,
    Global,
    Save,
}

fn deserialize_arg<'de, D>(deserializer: D) -> Result<Arg, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Arg::deserialize(deserializer).unwrap_or(Arg::Any))
}

fn deserialize_upper_camel<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.to_upper_camel_case())
}
