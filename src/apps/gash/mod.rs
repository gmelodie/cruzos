use crate::{keyboard::getc, prelude::*};

pub struct Gash {}

impl Gash {
    pub fn new() -> Self {
        Gash {}
    }
    pub async fn run(&self) {
        let mut input = String::new();
        loop {
            print!("root@cruzos # ");
            while let c = getc().await {
                if c == 8 as char {
                    stdout().backspace();
                    continue;
                }

                print!("{c}");
                if c == '\n' {
                    break;
                }
                input.push(c);
            }

            // scanf(&mut input).await;
            // TODO: run input
            input.clear();
        }
    }
}
