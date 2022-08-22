use game::Game;
use std::env;
use strategy::{EntropyStrategy, Strategy};
use words::{Pattern, Wordlist};

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

    let mut game = Game::init(wordlist, &EntropyStrategy::init);
    game.choose_word();
}
