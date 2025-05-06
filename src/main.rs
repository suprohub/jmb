pub mod assets;
pub mod bytecode;
pub mod generated;
pub mod module;
use std::{fs::File, io::BufReader};

use bytecode::compile;

fn main() {
    simple_logger::init().unwrap();
    let module: module::Module =
        serde_json::from_reader(BufReader::new(File::open("pr1.json").unwrap())).unwrap();
    log::info!("{:?}", module);
    //println!("{:?}", get_assets());

    for line in &module.handlers {
        for op in &line.operations {
            log::info!("{}", op.action as u16)
        }
    }

    log::info!("batatacode go now");
    log::info!("{:?}", compile(module));
}
