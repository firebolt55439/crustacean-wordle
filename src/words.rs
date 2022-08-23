use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::game::Guess;
use crate::game::GuessOutcome;

/// Allowed word length.
const WORD_LENGTH: usize = 5;

/// Whether or not to strip out words that have a frequency score of 0
/// from the wordlist.
const IGNORE_ZERO_FREQ_WORDS: bool = true;

/// For a specific word character position, this object records whether or not
/// the position is known to contain a specific character or known to not
/// contain a set of characters.
#[derive(Debug, Clone)]
pub enum PlaceConstraint {
    IsChar(char),
    IsNotChars(Vec<char>),
}

/// Stores the accumulation of knowledge gained over a Wordle game, specifically
/// the characters that do not exist in the word and the positional character
/// constraints.
#[derive(Default)]
pub struct Pattern {
    pub disallowed: Vec<char>,
    pub constraints: HashMap<usize, PlaceConstraint>,
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

        Pattern {
            disallowed,
            constraints,
        }
    }
}

#[derive(Default)]
pub struct Word {
    word: Vec<char>,
}

pub type WordPtr = Arc<Word>;

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_word())
    }
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

    /// Return the outcome of the given guess against this word.
    pub fn outcome_of_guess(&self, guess: WordPtr) -> Box<Guess> {
        let chars = guess.word.clone();
        let mut outcomes: Vec<GuessOutcome> = vec![];
        for (idx, ch) in chars.iter().enumerate() {
            let mut outcome = GuessOutcome::Gray;
            if self.has_letter(ch) {
                if let Some(actual) = self.word.get(idx) {
                    if actual == ch {
                        outcome = GuessOutcome::Green;
                    }
                } else {
                    outcome = GuessOutcome::Yellow;
                }
            }
            outcomes.push(outcome);
        }

        Box::new(Guess {
            guess: chars,
            outcome: outcomes,
        })
    }

    /// Getter for the word as a string.
    pub fn get_word(&self) -> String {
        self.word.iter().collect()
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

pub trait HasWords {
    fn possible_words(&self) -> &[WordPtr];

    /// Return a random word if non-empty or None.
    fn random_word(&self) -> Option<WordPtr> {
        let mut rng = thread_rng();
        let words = self.possible_words();
        words.choose(&mut rng).cloned()
    }

    /// Returns the unweighted entropy of this distribution (i.e. the -log2 of the cardinality of
    /// the remaining guessing space).
    fn unweighted_entropy(&self) -> f64 {
        (self.possible_words().len() as f64).log2()
    }
}

pub trait HasWordScores: HasWords {
    fn possible_scores(&self) -> &[f64];

    /// Returns the unweighted entropy of this distribution (i.e. the -log2 of the cardinality of
    /// the remaining guessing space).
    fn weighted_entropy(&self) -> f64 {
        let mut sum: f64 = 0.0_f64;
        let scores = self.possible_scores();
        let num = scores.len() as f64;
        for score in scores {
            // println!("Score: {}", score);
            let mut probability = 1.0_f64 / num;
            probability *= (100_f64 + score) / 100_f64; // higher z-score, higher probability
            sum += -probability * probability.log2();
        }
        sum
    }
}

pub trait CanPatternFilter: HasWordScores {
    /// Returns a `SubWordlist` of the words matching the given `Pattern`.
    fn filter_pattern(&self, pattern: &Pattern) -> Arc<SubWordlist> {
        let words = self.possible_words();
        let scores = self.possible_scores();
        let indices: Vec<usize> = words
            .iter()
            .enumerate()
            .filter_map(|(idx, word)| {
                if word.matches(pattern) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        Arc::new(SubWordlist {
            words: indices
                .iter()
                .filter_map(|idx| words.get(*idx))
                .cloned()
                .collect(),
            scores: indices
                .iter()
                .filter_map(|idx| scores.get(*idx))
                .cloned()
                .collect(),
        })
    }
}

/// The Wordlist object contains the list of all valid words and associated frequencies.
#[derive(Default)]
pub struct Wordlist {
    words: Vec<WordPtr>,
    scores: Vec<f64>,
}

pub type WordlistPtr = Arc<Wordlist>;

impl HasWords for Wordlist {
    fn possible_words(&self) -> &[WordPtr] {
        &self.words
    }
}

impl HasWordScores for Wordlist {
    fn possible_scores(&self) -> &[f64] {
        &self.scores
    }
}

impl CanPatternFilter for Wordlist {}

impl Wordlist {
    /// Initialize a `Wordlist` from the wordlist at the file path `path`. The file
    /// is assumed to have multiple space-separated columns. This function
    /// requires that the first column corresponds to the word and the last column
    /// corresponds to a nonnegative score, such that higher scores indicate the
    /// word more frequently occurs.
    pub fn init(path: &String) -> Arc<Self> {
        println!("Loading wordlist...");
        let file =
            File::open(path).expect(format!("Could not read wordlist at path '{}'", path).as_str());

        let mut words: Vec<WordPtr> = vec![];
        let mut scores: Vec<f64> = vec![];

        let lines = BufReader::new(file).lines();

        lines.filter_map(|line| line.ok()).for_each(|line| {
            let mut columns = line.split_whitespace();
            let word = columns.next().expect("Could not read word from column");
            let score: f64 = columns.last().unwrap_or("0").parse().unwrap();

            // Check length of word
            if word.len() != WORD_LENGTH {
                return;
            }

            // If enabled, filter out word if it has a nonpositive frequency score
            if IGNORE_ZERO_FREQ_WORDS && score <= 0.0_f64 {
                return;
            }

            words.push(Arc::new(Word::from(word)));
            scores.push(score);
        });

        let words = words;
        let scores = Wordlist::normalize_scores(scores);

        println!("Loaded wordlist.");

        Arc::new(Wordlist { words, scores })
    }

    /// Normalize scores in the given vector by mapping them to a function of
    /// the base-10 logarithm of their z-scores.
    fn normalize_scores(scores: Vec<f64>) -> Vec<f64> {
        if scores.is_empty() {
            return vec![];
        }

        let num = scores.len() as f64;
        let min = scores.iter().fold(f64::INFINITY, |a, b| f64::min(a, *b));
        let max = scores
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| f64::max(a, *b));
        let sum: f64 = scores.iter().sum();
        let mean = sum / num;
        let variance: f64 = scores
            .iter()
            .map(|&val| (val - mean) * (val - mean))
            .sum::<f64>()
            / num;
        let stddev = variance.sqrt();

        println!(
            "N: {}, sum: {}, min: {}, max: {}, mean: {}, variance: {}, stddev: {}",
            num, sum, min, max, mean, variance, stddev
        );

        let z_scores: Vec<f64> = scores
            .into_iter()
            .map(|score| {
                let z_score: f64 = (score - mean) / stddev;
                z_score
            })
            .collect();

        let min = z_scores.iter().fold(f64::INFINITY, |a, b| f64::min(a, *b));
        z_scores
            .into_iter()
            .map(|score| (score + min.abs() * 2_f64).log2())
            .collect()
    }
}

#[derive(Default)]
pub struct SubWordlist {
    words: Vec<WordPtr>,
    scores: Vec<f64>,
}

impl HasWords for SubWordlist {
    fn possible_words(&self) -> &[WordPtr] {
        &self.words
    }
}

impl HasWordScores for SubWordlist {
    fn possible_scores(&self) -> &[f64] {
        &self.scores
    }
}

impl CanPatternFilter for SubWordlist {}

impl SubWordlist {}
