use crate::prelude::*;

pub struct Crash {}

impl Crash {
    pub fn new() -> Self {
        Crash {}
    }
    pub fn run(&self) {
        let mut input = String::new();
        loop {
            print!("root@crash # ");
            scanf(&mut input);
            // TODO: run input
            print!("{input}");
            input.clear();
        }
    }
}
