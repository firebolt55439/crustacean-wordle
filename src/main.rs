use std::env;
use words::Wordlist;

mod game;
mod strategy;
mod words;

fn main() {
    let args: Vec<String> = env::args().collect();
    let wordlist = Wordlist::init(
        &args
            .get(1)
            .expect("Wordlist file path parameter required as first command-line argument"),
    );
}
