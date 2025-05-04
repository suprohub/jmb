use std::fs::{self};

use serde::{Deserialize, Serialize};

use crate::generated::{
    ActionId, ActionObject, ActionType, ArgType, EventId, GameValueId, ValueType,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub id: EventId,
    pub cancellable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameValue {
    pub id: GameValueId,
    #[serde(rename = "type")]
    pub value_type: ValueType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Action {
    pub id: ActionId,
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub object: ActionObject,
    pub args: Vec<Arg>,
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
        // Skip id
        //let _ = String::deserialize(deserializer)?;

        // Parse type
        match ArgType::deserialize(deserializer)? {
            _ => {}
        }

        Ok(Arg::Any)
    }
}

pub fn get_assets() -> Result<(Vec<Event>, Vec<GameValue>, Vec<Action>), Box<dyn std::error::Error>>
{
    Ok((
        serde_json::from_str(&fs::read_to_string("assets/events.json")?)?,
        serde_json::from_str(&fs::read_to_string("assets/game_values.json")?)?,
        serde_json::from_str(&fs::read_to_string("assets/actions.json")?)?,
    ))
}
