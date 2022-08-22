use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::game::Guess;
use crate::game::GuessOutcome;

/// For a specific word character position, this object records whether or not
/// the position is known to contain a specific character or known to not
/// contain a set of characters.
#[derive(Clone)]
pub enum PlaceConstraint {
    IsChar(char),
    IsNotChars(Vec<char>),
}

/// Stores the accumulation of knowledge gained over a Wordle game, specifically
/// the characters that do not exist in the word and the positional character
/// constraints.
#[derive(Default)]
pub struct Pattern {
    disallowed: Vec<char>,
    constraints: HashMap<usize, PlaceConstraint>,
}

impl Pattern {
    /// Update the knowledge contained within this `Pattern` with the outcome
    /// of `guess`.
    pub fn ingest(&self, guess: &Guess) -> Self {
        let mut disallowed = self.disallowed.clone();

        let mut constraints: HashMap<usize, PlaceConstraint> = HashMap::new();
        constraints.reserve(self.constraints.len());
        let constraints_kvs = self
            .constraints
            .iter()
            .map(|(a, b)| (a.to_owned(), b.to_owned()));
        constraints.extend(constraints_kvs);

        for (idx, (ch, outcome)) in guess.paired_iter().enumerate() {
            match outcome {
                GuessOutcome::Gray => {
                    if !disallowed.contains(ch) {
                        disallowed.push(ch.to_owned());
                    }
                }
                GuessOutcome::Yellow => {
                    if let Some(cons) = constraints.get_mut(&idx) {
                        match cons {
                            PlaceConstraint::IsChar(_) => {}
                            PlaceConstraint::IsNotChars(chars) => {
                                if !chars.contains(ch) {
                                    chars.push(ch.to_owned());
                                }
                            }
                        }
                    } else {
                        constraints.insert(idx, PlaceConstraint::IsNotChars(vec![ch.to_owned()]));
                    }
                }
                GuessOutcome::Green => {
                    constraints.insert(idx, PlaceConstraint::IsChar(ch.to_owned()));
                }
            }
        }

        Pattern::default()
    }
}

pub struct Word {
    word: Vec<char>,
}

impl Word {
    /// Whether or not the word contains character `ch`.
    fn has_letter(&self, ch: &char) -> bool {
        self.word.contains(ch)
    }

    /// Whether or not the word obeys the `PlaceConstraint` for the
    /// given `usize` index.
    fn obeys_constraint(&self, cons: (&usize, &PlaceConstraint)) -> bool {
        let (idx, cons) = cons;
        let value = self
            .word
            .get(*idx)
            .expect("Constraint index out of bounds!");

        match cons {
            PlaceConstraint::IsChar(ch) => ch == value,
            PlaceConstraint::IsNotChars(chars) => !chars.contains(value),
        }
    }

    /// Whether or not this word matches the `Pattern` given in `pattern`.
    pub fn matches(&self, pattern: &Pattern) -> bool {
        if pattern.disallowed.iter().any(|ch| self.has_letter(ch)) {
            return false;
        }

        if !pattern
            .constraints
            .iter()
            .all(|cons| self.obeys_constraint(cons))
        {
            return false;
        }

        true
    }
}

impl From<String> for Word {
    fn from(str: String) -> Self {
        Word {
            word: str.chars().collect(),
        }
    }
}

impl From<&str> for Word {
    fn from(str: &str) -> Self {
        Word {
            word: str.chars().collect(),
        }
    }
}

pub struct Wordlist {
    words: Vec<Word>,
    scores: Vec<f64>,
}

impl Wordlist {
    /// Initialize a `Wordlist` from the wordlist at the file path `path`. The file
    /// is assumed to have multiple space-separated columns. This function
    /// requires that the first column corresponds to the word and the last column
    /// corresponds to a nonnegative score, such that higher scores indicate the
    /// word more frequently occurs.
    pub fn init(path: &String) -> Self {
        let file =
            File::open(path).expect(format!("Could not read wordlist at path '{}'", path).as_str());

        let mut words: Vec<Word> = vec![];
        let mut scores: Vec<f64> = vec![];

        let lines = BufReader::new(file).lines();

        lines.filter_map(|line| line.ok()).for_each(|line| {
            let mut columns = line.split(" ");
            let word = columns.next().expect("Could not read word from column");
            let score: f64 = columns
                .filter(|s| !s.is_empty())
                .last()
                .unwrap_or("0")
                .parse()
                .unwrap();

            words.push(Word::from(word));
            scores.push(score);
        });

        Wordlist { words, scores }
    }
}
