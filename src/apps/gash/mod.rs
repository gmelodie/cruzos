#[allow(unused)]
use alloc::{borrow::ToOwned, string::ToString};

use crate::{keyboard::getc, prelude::*};

pub struct Gash {}

fn echo(args: Vec<&str>) -> Result<()> {
    println!("{}", args.join(" "));
    Ok(())
}

fn ps(_args: Vec<&str>) -> Result<()> {
    // TODO: add flags
    // format!("{}", executor.task_names().join("\n"));
    Ok(())
}

fn parse_cmd(input: &str) -> (&str, Vec<&str>) {
    let mut iter = input.trim().split_ascii_whitespace();
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

            let (cmd, args) = parse_cmd(input.as_str());
            if let Err(msg) = match cmd {
                "echo" => echo(args),
                "ps" => ps(args),
                _ => err!("command not found: {cmd}"),
            } {
                println!("gash: {msg}");
            }
            input.clear();
        }
    }
}
