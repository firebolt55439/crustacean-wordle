use console::{style, Color, Term};
use std::{collections::BTreeMap, fmt::Display, iter::Zip, sync::Arc};

use crate::{
    strategy::{Strategy, StrategyVerbosity},
    words::{HasWords, WordPtr, WordlistPtr},
};

/// The maximum number of allowed guesses per game.
const ALLOWED_GUESSES_PER_GAME: usize = 6;

/// Represents the outcomes of a guess for a single character tile.
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum TileOutcome {
    Gray,
    Yellow,
    Green,
}

impl TileOutcome {
    pub fn color(&self) -> Color {
        match self {
            TileOutcome::Gray => Color::Color256(242), // Grey42 (#6c6c6c)
            TileOutcome::Green => Color::Green,
            TileOutcome::Yellow => Color::Yellow,
        }
    }
}

/// Represents a guess and its paired outcome (i.e. gray/green/yellow tiles).
#[derive(Default)]
pub struct Guess {
    pub guess: Vec<char>,
    pub outcome: Vec<TileOutcome>,
}

impl Display for Guess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (ch, outcome) in self.guess.iter().zip(self.outcome.iter()) {
            write!(
                f,
                "{}",
                style(ch.to_ascii_uppercase())
                    .fg(Color::White)
                    .bg(outcome.color())
                    .bright()
                    .bold()
            )?;
        }

        Ok(())
    }
}

impl Guess {
    pub fn paired_iter(
        &self,
    ) -> Zip<std::slice::Iter<'_, char>, std::slice::Iter<'_, TileOutcome>> {
        self.guess.iter().zip(self.outcome.iter())
    }
}

/// Represents the game state, including the chosen word, the past guesses, the
/// strategy being tested, the wordlist, and the history of strategy metrics.
pub struct Game {
    word: WordPtr,
    guesses: Vec<Box<Guess>>,
    answerlist: WordlistPtr,
    guesslist: WordlistPtr,
    history: Vec<BTreeMap<String, f64>>,
    strategy: Box<dyn Strategy>,
}

pub enum GameState {
    NotStarted,
    InProgress,
    GuesserVictory,
    GuesserDefeat,
    Invalid,
}

impl Game {
    /// Initializes a new Game with the given `wordlist` and strategy initialization function.
    pub fn init(
        guesslist: WordlistPtr,
        answerlist: WordlistPtr,
        strategy_init: &dyn Fn(WordlistPtr) -> Box<dyn Strategy>,
    ) -> Self {
        let mut game = Game {
            word: Arc::default(),
            guesses: vec![],
            answerlist,
            guesslist: guesslist.clone(),
            history: vec![],
            strategy: strategy_init(guesslist),
        };
        game.push_metrics();
        game
    }

    /// Choose a word at random from the answer list.
    #[allow(dead_code)]
    pub fn choose_random_word(&mut self) {
        self.word = self
            .answerlist
            .random_word()
            .expect("Could not choose random word from empty answer list!");
    }

    /// Choose the given word (must only be in the guess list, not answer list).
    #[allow(dead_code)]
    pub fn choose_word(&mut self, word: &str) {
        self.word = self
            .guesslist
            .get_word(word)
            .expect("Given word is not in guess list!");
    }

    /// Make a given guess.
    pub fn make_guess(&mut self, guess: WordPtr) {
        let outcome = self.word.outcome_of_guess(guess.clone());
        let guess = Box::new(Guess {
            guess: guess.get_word().chars().collect(),
            outcome,
        });
        self.strategy.register_guess(&guess);
        self.guesses.push(guess);
        self.push_metrics();
    }

    fn push_metrics(&mut self) {
        self.history.push(self.strategy.metrics());
    }

    /// Retrieve next guess from strategy.
    pub fn next_guess(&self) -> WordPtr {
        self.strategy.chosen_guess()
    }

    /// Pretty-print game state.
    pub fn pretty_print(&self) -> Result<(), std::io::Error> {
        let term = Term::stdout();

        term.write_line(
            &style("CRUSTACEAN WORDLE")
                .cyan()
                .bright()
                .bold()
                .underlined()
                .to_string(),
        )?;
        term.move_cursor_down(1)?;

        term.write_line(
            format!(
                "Game is {}.",
                style(match self.current_state() {
                    GameState::Invalid => "an invalid state",
                    GameState::NotStarted => "starting",
                    GameState::InProgress => "in progress",
                    GameState::GuesserDefeat | GameState::GuesserVictory => "over",
                })
                .bold()
            )
            .as_str(),
        )?;
        term.move_cursor_down(1)?;

        for (idx, guess) in self.guesses.iter().enumerate() {
            term.write_line(format!("#{}: {}", idx + 1, guess).as_str())?;
        }

        if self.guesses.is_empty() {
            term.write_line("No guesses yet.")?;
        }

        Ok(())
    }

    /// Return all allowed guesses according to the wordlist.
    pub fn allowed_guesses(&self) -> Vec<WordPtr> {
        self.guesslist.possible_words().to_vec()
    }

    /// Return all extant guesses according to the strategy.
    #[allow(dead_code)]
    pub fn extant_guesses(&self) -> &[WordPtr] {
        self.strategy.extant_guesses()
    }

    /// Retrieve the current game state.
    pub fn current_state(&self) -> GameState {
        if self.word.get_word().is_empty() {
            return GameState::NotStarted;
        }

        if self.guesses.is_empty() {
            return GameState::InProgress;
        }

        if let Some(last_guess) = self.guesses.last() {
            if last_guess
                .outcome
                .iter()
                .all(|item| item == &TileOutcome::Green)
            {
                GameState::GuesserVictory
            } else if self.guesses.len() >= ALLOWED_GUESSES_PER_GAME {
                GameState::GuesserDefeat
            } else {
                GameState::InProgress
            }
        } else {
            GameState::Invalid
        }
    }

    /// Return if this game has ended (i.e. a victory or defeat for the guesser).
    pub fn is_over(&self) -> bool {
        match self.current_state() {
            GameState::GuesserDefeat | GameState::GuesserVictory => true,
            _ => false,
        }
    }

    /// Set strategy verbosity.
    pub fn set_verbosity(&mut self, verbosity: StrategyVerbosity) {
        self.strategy.set_verbosity(verbosity)
    }

    /// Retrieve wordlist.
    pub fn get_wordlist(&self) -> WordlistPtr {
        self.guesslist.clone()
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let divider = "+---".to_string().repeat(20);
        writeln!(f)?;
        writeln!(f, "{}", divider)?;
        writeln!(f, "Chosen word: {}", self.word)?;
        writeln!(f, "# Guesses: {}", self.guesses.len())?;
        writeln!(
            f,
            "# Allowed Guesses: {} (entropy: {})",
            self.allowed_guesses().len(),
            self.guesslist.unweighted_entropy()
        )?;

        writeln!(f, "{}", divider)?;
        for (idx, guess) in self.guesses.iter().enumerate() {
            let guessed_word: String = guess.guess.iter().collect();
            writeln!(
                f,
                "Guess #{}: {} [{:?}]",
                idx + 1,
                guessed_word,
                guess.outcome
            )?;
        }

        writeln!(f, "{}", divider)?;
        self.strategy.pretty_print(&self.history);

        writeln!(f, "{}", divider)
    }
}
