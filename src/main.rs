pub mod assets;
pub mod generated;
pub mod module;
use std::{fs::File, io::BufReader};

fn main() {
    let module: module::Module =
        serde_json::from_reader(BufReader::new(File::open("pr1.json").unwrap())).unwrap();
    println!("{:?}", module);
    //println!("{:?}", get_assets());

    for line in module.handlers {
        for op in line.operations {
            println!("{}", op.action as u16)
        }
    }
}
