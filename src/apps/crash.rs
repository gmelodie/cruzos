use crate::prelude::*;

pub struct Crash {}

impl Crash {
    pub fn new() -> Self {
        Crash {}
    }
    pub async fn run(&self) {
        let mut input = String::new();
        loop {
            print!("root@crash # ");
            scanf(&mut input).await;
            // TODO: run input
            input.clear();
        }
    }
}
