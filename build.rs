use heck::{ToPascalCase, ToSnakeCase};
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;

fn justmc_skill_issue(name: &str) -> String {
    let snake = name.to_snake_case();
    let re = Regex::new(r"([a-zA-Z])(\d+)([a-zA-Z])").unwrap();
    re.replace_all(&snake, |caps: &regex::Captures| {
        format!("{}_{}{}", &caps[1], &caps[2], &caps[3])
    })
    .to_string()
}

fn generate_enum(
    f: &mut File,
    enum_name: &str,
    variants: impl Itertools<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(f)?;
    writeln!(
        f,
        "#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]"
    )?;
    writeln!(f, "#[serde(rename_all = \"snake_case\")]")?;
    writeln!(f, "#[repr(u16)]")?;
    writeln!(f, "pub enum {} {{", enum_name)?;

    for variant in variants.sorted() {
        if variant.chars().any(|c| c.is_ascii_digit()) {
            writeln!(
                f,
                "\t#[serde(rename = \"{}\")]",
                justmc_skill_issue(&variant)
            )?;
        }

        writeln!(f, "\t{},", variant.to_pascal_case())?;
    }

    writeln!(f, "}}")?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct RawEvent {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct RawGameValue {
    id: String,
    #[serde(rename = "type")]
    value_type: String,
}

#[derive(Serialize, Deserialize)]
struct RawAction {
    id: String,
    #[serde(rename = "type")]
    action_type: String,
    object: String,
    args: Vec<RawArg>,
}

#[derive(Serialize, Deserialize)]
struct RawArg {
    #[serde(rename = "type")]
    arg_type: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut f = File::create("src/generated.rs")?;

    let events: Vec<RawEvent> =
        serde_json::from_str(&fs::read_to_string("assets/events.json")?).unwrap();
    let mut ids = HashSet::with_capacity(events.len());

    for event in events {
        ids.insert(event.id);
    }

    generate_enum(&mut f, "EventId", ids.into_iter())?;

    let game_values: Vec<RawGameValue> =
        serde_json::from_str(&fs::read_to_string("assets/game_values.json")?).unwrap();
    let mut ids = HashSet::with_capacity(game_values.len());
    let mut typs = HashSet::new();

    for value in game_values {
        ids.insert(value.id);
        typs.insert(value.value_type);
    }

    generate_enum(&mut f, "GameValueId", ids.into_iter())?;
    generate_enum(&mut f, "ValueType", typs.into_iter())?;

    let actions: Vec<RawAction> =
        serde_json::from_str(&fs::read_to_string("assets/actions.json")?).unwrap();
    let mut ids = HashSet::new();
    let mut typs = HashSet::new();
    let mut objs = HashSet::new();
    let mut arg_types = HashSet::new();

    for action in actions {
        ids.insert(action.id);
        typs.insert(action.action_type);
        objs.insert(action.object);

        for arg in action.args {
            arg_types.insert(arg.arg_type);
        }
    }

    generate_enum(&mut f, "ActionIdWants11Bits", ids.into_iter())?;
    generate_enum(&mut f, "ActionType", typs.into_iter())?;
    generate_enum(&mut f, "ActionObject", objs.into_iter())?;
    generate_enum(&mut f, "ArgType", arg_types.into_iter())?;

    Ok(())
}
