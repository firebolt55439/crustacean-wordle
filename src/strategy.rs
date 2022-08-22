use std::rc::Rc;

use crate::{
    game::Guess,
    words::{CanPatternFilter, Pattern, Word, Wordlist},
};

pub trait Strategy {
    /// All the guesses this strategy will consider making.
    fn possible_guesses(&self) -> Vec<Rc<Word>>;

    /// A callback function for the game to register a new `Guess`
    /// with this strategy.
    fn register_guess(&mut self, guess: &Guess);
}

pub struct EntropyStrategy {
    knowledge: Pattern,
    extant: Rc<dyn CanPatternFilter>,
}

impl Strategy for EntropyStrategy {
    fn possible_guesses(&self) -> Vec<Rc<Word>> {
        self.extant.possible_words().to_vec()
    }

    fn register_guess(&mut self, guess: &Guess) {
        self.knowledge = self.knowledge.ingest(guess);
        self.extant = self.extant.filter_pattern(&self.knowledge);
    }
}

impl EntropyStrategy {
    /// Initializes a new Strategy with the given Game.
    pub fn init(wordlist: Rc<Wordlist>) -> Box<dyn Strategy> {
        Box::new(EntropyStrategy {
            knowledge: Pattern::default(),
            extant: wordlist.clone(),
        })
    }
}
