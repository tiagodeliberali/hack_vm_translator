use std::fs;
use std::{env, path::Path};

mod builder;
mod parser;

use crate::builder::build_content;
use crate::parser::{initial_data, parse_content};

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("Please supply a folder or file name");
    let name = Path::new(path).file_name().unwrap().to_str().unwrap();

    let mut result: Vec<String> = Vec::new();
    let output: String;

    if path.ends_with(".vm") {
        result.extend(parse_file(path));
        output = path.replace(".vm", ".asm");
    } else {
        result.extend(initial_data());

        let file_list = fs::read_dir(path).unwrap();

        for file in file_list {
            let file_path_buff = file.unwrap().path();
            let file_path = file_path_buff.to_str().unwrap();
            let file_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();

            if file_name.ends_with(".vm") {
                result.extend(parse_file(&file_path));
            }
        }
        output = format!("{}/{}.asm", path, name);
    }

    fs::write(output, result.join("\r\n")).expect("Something failed on write file to disk");
}

fn parse_file(file_path: &str) -> Vec<String> {
    let content = fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let lines = build_content(content);

    let filename = Path::new(file_path).file_name().unwrap().to_str().unwrap();

    parse_content(lines, filename.replace(".vm", ""))
}
