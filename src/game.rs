use std::iter::Zip;

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
