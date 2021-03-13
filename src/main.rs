use std::{env, path::Path};
use std::fs;

mod builder;
mod parser;

use crate::builder::build_content;
use crate::parser::parse_content;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = args.get(1).expect("Please supply a filename");

    let content = fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let lines = build_content(content);

    let filename = Path::new(file_path).file_name().unwrap().to_str().unwrap();

    let result = parse_content(lines, filename.replace(".vm", ""));

    fs::write(file_path.replace(".vm", ".asm"), result.join("\r\n"))
        .expect("Something failed on write file to disk");
}
