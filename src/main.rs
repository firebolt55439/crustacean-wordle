use clap::Parser;
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect};
use game::{Game, GameState};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::{path::PathBuf, sync::atomic::AtomicU64};
use strategy::EntropyStrategy;
use words::{HasWords, Wordlist, WordlistPtr};

mod bitmask;
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

    /// Turn debugging information on
    #[clap(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Run a benchmark
    #[clap(short, long, action = clap::ArgAction::Count)]
    benchmark: u8,
}

fn human_repl(game: &mut Game) -> Result<(), std::io::Error> {
    let term = Term::stdout();
    term.set_title("Crustacean Wordle");

    term.clear_screen()?;
    game.pretty_print()?;

    let wordlist = game.get_wordlist();
    let word_slice = wordlist.get_word_slice();

    while !game.is_over() {
        term.write_line("")?;

        if Confirm::with_theme(&ColorfulTheme::default())
            .default(true)
            .with_prompt("Do you want to make a guess?")
            .interact()?
        {
            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("What is your guess?")
                .with_visible_term_rows(10_usize)
                .items(word_slice)
                .report(true)
                .default(0)
                .interact()?;

            let word = word_slice[selection].clone();
            game.make_guess(word);
        } else {
            term.write_line("Consulting strategy for next guess.")?;

            let guess = game.next_guess().ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not retrieve guess strategy!",
            ))?;
            game.make_guess(guess);
        }

        term.clear_screen()?;
        game.pretty_print()?;
    }

    term.write_line("")?;
    term.write_line("Thanks for playing!")
}

fn benchmark(answer_list: WordlistPtr, guess_list: WordlistPtr) -> Result<(), std::io::Error> {
    let term = Term::stdout();
    term.set_title("Crustacean Wordle");

    term.clear_screen()?;
    term.write_line(style("Benchmarking").bold().to_string().as_str())?;

    // Cache the first guess
    term.write_line("")?;
    term.write_line("Caching first guess...")?;

    let mut game = Game::init(
        guess_list.clone(),
        answer_list.clone(),
        &EntropyStrategy::init,
    );
    game.set_verbosity(strategy::StrategyVerbosity::PrettyPrint);

    let first_guess = game.next_guess().ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Could not compute first guess",
    ))?;

    // Benchmark possible answers in parallel
    term.write_line("")?;
    term.write_line("Benchmarking in parallel...")?;

    let possible_answers = answer_list.possible_words();
    let num_words = possible_answers.len() as u64;

    let pb = ProgressBar::new(num_words);
    let sty = ProgressStyle::with_template(
            "[{spinner:.green} {elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg} (eta {eta})",
        )
        .unwrap()
        .progress_chars("##-");
    pb.set_style(sty);
    pb.enable_steady_tick(Duration::from_millis(250));
    term.hide_cursor()?;

    let num_guesses_total = AtomicU64::new(0);
    let num_failed = AtomicU64::new(0);

    possible_answers.par_iter().for_each(|word| {
        let mut game = Game::init(
            guess_list.clone(),
            answer_list.clone(),
            &EntropyStrategy::init,
        );
        game.set_verbosity(strategy::StrategyVerbosity::Silent);
        game.choose_word(&word.get_word());
        game.make_guess(first_guess.clone());

        while !game.is_over() {
            let guess = game.next_guess().expect("Could not compute guess!");
            game.make_guess(guess);
        }

        pb.inc(1);

        if game.current_state() != GameState::GuesserVictory {
            num_failed.fetch_add(1, Ordering::SeqCst);
        } else {
            num_guesses_total.fetch_add(game.num_guesses() as u64, Ordering::SeqCst);
        }
    });

    let num_guesses_total = num_guesses_total.load(Ordering::SeqCst);
    let num_failed = num_failed.load(Ordering::SeqCst);

    term.show_cursor()?;
    term.write_line("")?;
    term.write_line(
        format!(
            "Took {} total guesses to guess {} of {} words successfully (avg: {}/word)",
            num_guesses_total,
            num_words - num_failed,
            num_words,
            (num_guesses_total as f64) / (num_words as f64)
        )
        .as_str(),
    )
}

fn main() {
    let args = Args::parse();
    let answer_list = Wordlist::init(&args.answer_list);
    let guess_list = Wordlist::init(&args.guess_list);

    if args.benchmark != 0 {
        benchmark(answer_list, guess_list).unwrap();
    } else {
        let mut game = Game::init(guess_list, answer_list, &EntropyStrategy::init);
        game.choose_random_word();

        if args.debug != 0 {
            game.set_debug(&true);
            game.set_verbosity(strategy::StrategyVerbosity::Debug);
        } else {
            game.set_debug(&false);
            game.set_verbosity(strategy::StrategyVerbosity::PrettyPrint);
        }

        human_repl(&mut game).unwrap();

        if args.debug != 0 {
            let term = Term::stdout();
            term.write_line("").unwrap();
            term.write_line(format!("{}", game).as_str()).unwrap();
        }
    }
}
