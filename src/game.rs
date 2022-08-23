use std::{collections::HashMap, iter::Zip, sync::Arc};

use crate::{
    strategy::Strategy,
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

/// Represents a guess and its paired outcome (i.e. gray/green/yellow tiles).
#[derive(Default)]
pub struct Guess {
    pub guess: Vec<char>,
    pub outcome: Vec<TileOutcome>,
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
    wordlist: WordlistPtr,
    history: Vec<HashMap<String, f64>>,
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
        wordlist: WordlistPtr,
        strategy_init: &dyn Fn(WordlistPtr) -> Box<dyn Strategy>,
    ) -> Self {
        let mut game = Game {
            word: Arc::default(),
            guesses: vec![],
            wordlist: wordlist.clone(),
            history: vec![],
            strategy: strategy_init(wordlist),
        };
        game.push_metrics();
        game
    }

    /// Choose a word at random from the wordlist.
    pub fn choose_word(&mut self) {
        self.word = self
            .wordlist
            .random_word()
            .expect("Could not choose word from empty list!");
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
    pub fn pretty_print(&self) {
        let divider = "+---".to_string().repeat(20);
        println!();
        println!("{}", divider);
        println!("Chosen word: {}", self.word);
        println!("# Guesses: {}", self.guesses.len());
        println!(
            "# Allowed Guesses: {} (entropy: {})",
            self.allowed_guesses().len(),
            self.wordlist.unweighted_entropy()
        );

        println!("{}", divider);
        for (idx, guess) in self.guesses.iter().enumerate() {
            let guessed_word: String = guess.guess.iter().collect();
            println!("Guess #{}: {} [{:?}]", idx + 1, guessed_word, guess.outcome);
        }

        println!("{}", divider);
        self.strategy.pretty_print(&self.history);

        println!("{}", divider);
    }

    /// Return all allowed guesses according to the wordlist.
    pub fn allowed_guesses(&self) -> Vec<WordPtr> {
        self.wordlist.possible_words().to_vec()
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
}
