use game::Game;
use std::env;
use strategy::EntropyStrategy;
use words::{HasWords, Wordlist};

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

    let mut game = Game::init(wordlist.clone(), &EntropyStrategy::init);
    game.choose_word();

    while !game.is_over() {
        let guess = game.next_guess();
        println!("Guessing: {}", guess.get_word());
        game.make_guess(guess);
        game.pretty_print();
    }

    // let guess = wordlist
    //     .random_word()
    //     .expect("Could not make guess from empty wordlist");
    // println!("Guessing: {}", guess.get_word());

    // game.make_guess(guess);
    // game.pretty_print();
}
