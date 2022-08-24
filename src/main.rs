use clap::Parser;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect};
use game::Game;
use std::path::PathBuf;
use strategy::EntropyStrategy;
use words::Wordlist;

mod game;
mod strategy;
mod words;

/// Wordle for Rustaceans.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The wordlist of possible guesses that can be made
    #[clap(short, long, value_parser, value_name = "FILE")]
    guess_list: PathBuf,

    /// The wordlist of possible answers to randomly choose from
    #[clap(short, long, value_parser, value_name = "FILE")]
    answer_list: PathBuf,
}

fn human_repl(game: &mut Game) -> Result<(), std::io::Error> {
    let term = Term::stdout();
    term.set_title("Crustacean Wordle");

    term.clear_screen()?;
    game.pretty_print()?;

    let wordlist = game.get_wordlist();
    let word_slice = wordlist.get_word_slice();

    while !game.is_over() {
        term.move_cursor_down(1)?;

        if Confirm::with_theme(&ColorfulTheme::default())
            .default(true)
            .with_prompt("Do you want to make a guess?")
            .interact()?
        {
            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("What is your guess?")
                .items(word_slice)
                .report(true)
                .default(0)
                .interact()?;

            let word = word_slice[selection].clone();
            game.make_guess(word);
        } else {
            term.write_line("Consulting strategy for next guess.")?;
            game.make_guess(game.next_guess());
        }

        term.clear_screen()?;
        game.pretty_print()?;
    }

    term.move_cursor_down(1)?;
    term.write_line(format!("{}", game).as_str())?;

    term.move_cursor_down(1)?;
    term.write_line("Thanks for playing!")
}

fn main() {
    let args = Args::parse();
    let answer_list = Wordlist::init(&args.answer_list);
    let guess_list = Wordlist::init(&args.guess_list);

    let mut game = Game::init(guess_list, answer_list, &EntropyStrategy::init);
    game.choose_random_word();
    game.set_verbosity(strategy::StrategyVerbosity::PrettyPrint);

    human_repl(&mut game).unwrap();

    /*
    while !game.is_over() {
        let guess = game.next_guess();
        println!("Guessing: {}", guess.get_word());
        game.make_guess(guess);
        game.pretty_print().expect("Could not print game state!");
    }
    */

    // let guess = wordlist
    //     .random_word()
    //     .expect("Could not make guess from empty wordlist");
    // println!("Guessing: {}", guess.get_word());

    // game.make_guess(guess);
    // game.pretty_print();
}
