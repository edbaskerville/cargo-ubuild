extern crate strip_ansi_escapes;

use std::str::FromStr;
use std::process::{Command, Stdio, Child};
use std::io::{self, Write, BufRead, BufReader};
use std::env::args_os;

const JUMP_DELIM: &'static str = "--> ";

fn main() {
    Main::new().main();
}

struct Main {
    root: Option<String>,
}

impl Main {
    fn new() -> Self {
        Self {
            root: None
        }
    }
    
    fn main(&mut self) {
        let mut cmd = self.build_command();
        match cmd.spawn() {
            Ok(child) => {
                self.process_output(child);
            },
            Err(err) => {
                println!("An error occurred spawning child.");
                println!("{:?}", err);
            },
        }
    }
    
    fn build_command(&mut self) -> Command {
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
    
        cmd
    }

    fn process_output(&mut self, child: Child) {
        match child.stderr {
            Some(stderr) => {
            
                let mut reader = BufReader::new(stderr);
                for line_result in reader.lines() {
                    match line_result {
                        Ok(line) => {
                            self.process_line(line);
                        },
                        Err(err) => {
                            panic!("Error: {:?}", err);
                        },
                    }
                }
            },
            None => {
                panic!("No standard error, weird.");
            },
        }
    }

    fn process_line(&mut self, line: String) {
        eprintln!("{}", line);
        if let Ok(line_stripped_bytes) = strip_ansi_escapes::strip(line) {
            if let Ok(line_stripped) = String::from_utf8(line_stripped_bytes) {
                if let Some(root_url_index) = line_stripped.find("file://") {
                    self.update_root(line_stripped, root_url_index);
                }
                else {
                    self.process_jump(line_stripped);
                }
            }
        }
    }

    fn update_root(&mut self, line: String, root_url_index: usize) {
        self.root = Some(line.get((root_url_index)..(line.len() - 1)).unwrap().to_string());
    }

    fn process_jump(&mut self, line: String) {
        let line_pieces: Vec<&str> = line.split(JUMP_DELIM).collect();
        if line_pieces.len() == 2 {
            eprint!("{}", " ".repeat(
                line_pieces.get(0).unwrap().len() + JUMP_DELIM.len()
            ));
            let link_str = line_pieces.get(1).unwrap();

            let link_str_pieces: Vec<&str> = link_str.split(':').collect();
            if link_str_pieces.len() == 3 {
                let path = link_str_pieces.get(0).unwrap();
                let line = u32::from_str(link_str_pieces.get(1).unwrap()).unwrap();
                let col = u32::from_str(link_str_pieces.get(2).unwrap()).unwrap();
                
                let url = if path.starts_with("/") {
                    Some(format!("file://{}", path))
                } else {
                    match self.root {
                        Some(ref root) => {
                            Some(format!("{}/{}", root, path))
                        },
                        None => None,
                    }
                };
        
                if let Some(ref url) = url {
                    eprintln!("txmt://open?url={}&line={}&column={}", url, line, col);
                }
            }
        }
    }
}
