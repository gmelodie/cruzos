#[allow(unused)]
use alloc::{borrow::ToOwned, string::ToString};

use crate::{keyboard::getc, prelude::*};

pub struct Gash {}

fn echo(args: Vec<&str>) -> Result<()> {
    println!("{}", args.join(" "));
    Ok(())
}

fn parse_cmd(input: &str) -> (&str, Vec<&str>) {
    // TODO: trim input
    let mut iter = input.split_ascii_whitespace();
    let (cmd, args): (&str, Vec<&str>) = (iter.next().unwrap_or(""), iter.collect());
    (cmd, args)
}

impl Gash {
    pub fn new() -> Self {
        Gash {}
    }
    pub async fn run(&self) {
        let mut input = String::new();
        loop {
            print!("root@cruzos # ");
            while let c = getc().await {
                // Handle backspace
                if c == 8 as char {
                    if let Some(_) = input.pop() {
                        stdout().backspace();
                    }
                    continue;
                }

                print!("{c}");
                if c == '\n' {
                    break;
                }
                input.push(c);
            }

            // TODO: run input
            let (cmd, args) = parse_cmd(input.as_str());
            if let Err(msg) = match cmd {
                "echo" => echo(args),
                _ => err!("command not found: {cmd}"),
            } {
                println!("gash: {msg}");
            }
            input.clear();
        }
    }
}
