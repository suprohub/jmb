pub mod assets;
pub mod generated;
pub mod module;
use std::fs;

use assets::get_assets;

fn main() {
    let module: module::Module =
        serde_json::from_str(&fs::read_to_string("supe.json").unwrap()).unwrap();
    println!("{:?}", module);
    //println!("{:?}", get_assets());
}
