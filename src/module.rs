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

#[derive(Debug, Serialize, Deserialize)]
pub struct Arg {
    name: String,
    #[serde(rename = "value")]
    arg_type: ArgType,
}
