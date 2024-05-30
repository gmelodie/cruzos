use crate::prelude::*;

pub struct Gash {}

impl Gash {
    pub fn new() -> Self {
        Gash {}
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
