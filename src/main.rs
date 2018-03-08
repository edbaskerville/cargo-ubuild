extern crate strip_ansi_escapes;

use std::str::FromStr;
use std::process::{Command, Stdio};
use std::io::{self, Write, BufRead, BufReader};
use std::env::args_os;

const JUMP_DELIM: &'static str = "--> ";

fn main() {
    let mut cmd = Command::new("cargo");
    let mut args = args_os();
    cmd.arg("build").arg("--color").arg("always");
    
    // TODO: make it this doesn't fail if you run command as "cargo-ubuild"
    args.next();
    args.next();
    for arg in args {
        println!("{:?}", arg);
        cmd.arg(arg);
    }
    cmd.stderr(Stdio::piped());
    match cmd.spawn() {
        Ok(mut child) => {
            match child.stderr {
                Some(stderr) => {
                    let mut root = String::from("file://");
                    
                    let mut reader = BufReader::new(stderr);
                    for line_result in reader.lines() {
                        match line_result {
                            Ok(line) => {
                                println!("{}", line);
                                if let Ok(line_stripped_bytes) = strip_ansi_escapes::strip(line) {
                                    if let Ok(line_stripped) = String::from_utf8(line_stripped_bytes) {
                                        if let Some(root_url_index) = line_stripped.find("file://") {
                                            root.clear();
                                            root.push_str(line_stripped.get((root_url_index)..(line_stripped.len() - 1)).unwrap());
                                        }
                                        else {
                                            let line_stripped_pieces: Vec<&str> = line_stripped.split(JUMP_DELIM).collect();
                                            if line_stripped_pieces.len() == 2 {
                                                print!("{}", " ".repeat(
                                                    line_stripped_pieces.get(0).unwrap().len() + JUMP_DELIM.len()
                                                ));
                                                let link_str = line_stripped_pieces.get(1).unwrap();
                                            
                                                let link_str_pieces: Vec<&str> = link_str.split(':').collect();
                                                if link_str_pieces.len() == 3 {
                                                    let path = link_str_pieces.get(0).unwrap();
                                                    let line = u32::from_str(link_str_pieces.get(1).unwrap()).unwrap();
                                                    let col = u32::from_str(link_str_pieces.get(2).unwrap()).unwrap();
                                                
                                                    println!("txmt://open?url={}/{}&line={}&column={}", root, path, line, col);
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            Err(err) => {
                                println!("{:?}", err);
                            },
                        }
                    }
                },
                None => {
                    println!("No standard error, weird.");
                },
            }
        },
        Err(err) => {
            println!("An error occurred spawning child.");
            println!("{:?}", err);
        },
    }
}
