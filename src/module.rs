use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::generated::{ActionId, ArgType};

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    handlers: Vec<Line>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    #[serde(rename = "type")]
    line_type: LineType,
    position: u32,
    operations: Vec<Op>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineType {
    Function,
    Event,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Op {
    action: ActionId,
    #[serde(rename = "values")]
    args: Vec<Arg>,
}

#[derive(Serialize, Debug)]
pub enum Arg {
    Any,
}

impl<'de> Deserialize<'de> for Arg {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Serialize, Deserialize)]
        struct Helper {
            name: String,
            value: Helper2,
        }

        #[derive(Serialize, Deserialize)]
        struct Helper2 {
            #[serde(rename = "type")]
            arg_type: Option<ArgType>,
            #[serde(flatten)]
            fields: HashMap<String, serde_json::Value>,
        }

        let Helper {
            name: _,
            value: Helper2 { arg_type, fields },
        } = Helper::deserialize(deserializer)?;

        if let Some(arg_type) = arg_type {
            match arg_type {
                ArgType::Any => {}
                ArgType::Array => {}
                _ => {}
            }
            Ok(Arg::Any)
        } else {
            Ok(Arg::Any)
        }
    }
}
