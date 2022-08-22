use std::{iter::Zip, rc::Rc};

use crate::{
    strategy::Strategy,
    words::{CanPatternFilter, HasWords, Pattern, Word, Wordlist},
};

pub enum GuessOutcome {
    Gray,
    Yellow,
    Green,
}

pub struct Guess {
    guess: Vec<char>,
    outcome: Vec<GuessOutcome>,
}

impl Guess {
    pub fn paired_iter(
        &self,
    ) -> Zip<std::slice::Iter<'_, char>, std::slice::Iter<'_, GuessOutcome>> {
        self.guess.iter().zip(self.outcome.iter())
    }
}

pub struct Game {
    word: Rc<Word>,
    guesses: Vec<Guess>,
    wordlist: Rc<Wordlist>,
    strategy: Box<dyn Strategy>,
}

impl Game {
    /// Initializes a new Game with the given `wordlist` and strategy initialization function.
    pub fn init(
        wordlist: Rc<Wordlist>,
        strategy_init: &dyn Fn(Rc<Wordlist>) -> Box<dyn Strategy>,
    ) -> Self {
        Game {
            word: Rc::default(),
            guesses: vec![],
            wordlist: wordlist.clone(),
            strategy: strategy_init(wordlist),
        }
    }

    /// Choose a word at random from the wordlist.
    pub fn choose_word(&mut self) {
        self.word = self
            .wordlist
            .random_word()
            .expect("Could not choose word from empty list!");
    }

    /// Make a given guess.
    pub fn make_guess(&mut self, guess: Rc<Word>) {
        //
    }

    /// Return all allowed guesses according to the wordlist.
    pub fn allowed_guesses(&self) -> Vec<Rc<Word>> {
        self.wordlist.possible_words().to_vec()
    }

    /// Return all possible guesses according to the strategy.
    pub fn possible_guesses(&self) -> Vec<Rc<Word>> {
        self.strategy.possible_guesses()
    }
}
