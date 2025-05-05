use std::fs::File;

use bitvec::{prelude::*, view::BitView};
use serde::{Deserialize, Serialize, Serializer};

use crate::module::Module;

pub fn compile(module: Module) -> String {
    serde_json::to_string(&module).unwrap()
}

pub fn decompile() {}

#[derive(Serialize, Deserialize)]
pub struct A {
    #[serde(serialize_with = "as_2bits")]
    b: u8,
}

pub fn as_2bits<S: Serializer, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize + BitStore,
    <T as BitStore>::Mem: Serialize,
{
    value.view_bits::<Lsb0>()[..2].serialize(serializer)
}

