use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;

use counter::Counter;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::bitmask::*;
use crate::game::TileOutcome;
use crate::pattern::{Pattern, PlaceConstraint};

/// Allowed word length (all words not of this length are filtered out in the Wordlist initializer).
pub const WORD_LENGTH: usize = 5;

/// Strip out words that have a frequency score of lower than this threshold.
// const FREQ_SCORE_THRESHOLD: f64 = 1000_f64;
const FREQ_SCORE_THRESHOLD: f64 = 0_f64;

#[derive(Default)]
pub struct Word {
    word: Vec<char>,
    letters_mask: LetterBitmask,
}

impl Debug for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = String::from_iter(self.word.iter());
        write!(f, "{}", s)
    }
}

pub type WordPtr = Arc<Word>;

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_word())
    }
}

impl Word {
    /// Whether or not the word contains character `ch`.
    #[inline(always)]
    fn has_letter(&self, ch: &char) -> bool {
        self.letters_mask & LetterBitmask::char_bitmask(ch) != 0
    }

    /// Number of occurrences of character `ch`.
    #[inline(always)]
    fn num_occurrences(&self, ch: &char) -> usize {
        self.word.iter().filter(|c| c == &ch).count()
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
            PlaceConstraint::IsNotChars(chars) => chars & LetterBitmask::char_bitmask(value) == 0,
        }
    }

    /// Whether or not this word matches the `Pattern` given in `pattern`.
    pub fn matches(&self, pattern: &Pattern) -> bool {
        if self.letters_mask & pattern.disallowed != 0 {
            return false;
        }

        for (ch, count) in &pattern.must_contain {
            if self.num_occurrences(ch) < *count {
                return false;
            }
        }

        for cons in &pattern.constraints {
            if !self.obeys_constraint(cons) {
                return false;
            }
        }

        true
    }

    /// Return the outcome of the given guess against this word.
    pub fn outcome_of_guess(&self, guess: WordPtr) -> Vec<TileOutcome> {
        debug_assert_eq!(guess.get_word().len(), self.get_word().len());

        let word_length_range = 0..guess.get_word().len();
        let mut outcomes: Vec<TileOutcome> = word_length_range
            .clone()
            .map(|_| TileOutcome::Gray)
            .collect();

        let mut counts = self.word.iter().collect::<Counter<_, i32>>();

        for idx in word_length_range {
            if self.word[idx] == guess.word[idx] {
                outcomes[idx] = TileOutcome::Green;
                counts[&&(self.word[idx])] -= 1;
            }
        }

        for (idx, ch) in guess.word.iter().enumerate() {
            if outcomes[idx] == TileOutcome::Gray && self.has_letter(ch) && counts[&ch] > 0 {
                counts[&ch] -= 1;
                outcomes[idx] = TileOutcome::Yellow;
            }
        }

        outcomes
    }

    /// Getter for the word as a string.
    pub fn get_word(&self) -> String {
        self.word.iter().collect()
    }
}

impl From<String> for Word {
    fn from(str: String) -> Self {
        let word: Vec<char> = str.chars().collect();
        let mask = LetterBitmask::compute_bitmask(word.iter());
        Word {
            word,
            letters_mask: mask,
        }
    }
}

impl From<&str> for Word {
    fn from(str: &str) -> Self {
        let word: Vec<char> = str.chars().collect();
        let mask = LetterBitmask::compute_bitmask(word.iter());
        Word {
            word,
            letters_mask: mask,
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
    pub fn init(path: &PathBuf) -> Arc<Self> {
        println!("Loading wordlist...");
        let file = File::open(path)
            .expect(format!("Could not read wordlist at path '{:?}'", path).as_str());

        let mut words: Vec<WordPtr> = vec![];
        let mut scores: Vec<f64> = vec![];

        let lines = BufReader::new(file).lines();

        lines.filter_map(|line| line.ok()).for_each(|line| {
            let mut columns = line.split_whitespace();
            let word = columns.next().expect("Could not read word from column");
            let score: f64 = columns.last().unwrap_or("0").parse().unwrap();

            // Check length of word.
            if word.len() != WORD_LENGTH {
                return;
            }

            // Filter out words with too low of a frequency score.
            if score < FREQ_SCORE_THRESHOLD {
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

    /// Find the given `word` in the list and return Some(match) if it
    /// is found, else None.
    pub fn get_word(&self, word: &str) -> Option<WordPtr> {
        self.words.iter().find(|&w| w.get_word() == word).cloned()
    }

    pub fn get_word_slice(&self) -> &[WordPtr] {
        &self.words
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_matches() {
        let pattern = Pattern {
            disallowed: LetterBitmask::compute_bitmask(vec!['a', 'b', 'c'].iter()),
            must_contain: HashMap::from_iter(vec![('d', 2_usize), ('e', 1_usize)]),
            constraints: HashMap::from_iter(vec![
                (0, PlaceConstraint::IsChar('d')),
                (
                    1,
                    PlaceConstraint::IsNotChars(vec!['d', 'e'].to_letter_bitmask()),
                ),
                (2, PlaceConstraint::IsChar('d')),
            ]),
        };

        assert!(!Word::from("ded").matches(&pattern));
        assert!(!Word::from("dede").matches(&pattern));
        assert!(!Word::from("ddde").matches(&pattern));
        assert!(!Word::from("dcde").matches(&pattern));
        assert!(Word::from("dfde").matches(&pattern));

        let pattern = Pattern {
            disallowed: LetterBitmask::compute_bitmask(vec!['y', 's', 'h', 'a', 't', 'z'].iter()),
            must_contain: HashMap::from_iter(vec![]),
            constraints: HashMap::from_iter(vec![
                (
                    0,
                    PlaceConstraint::IsNotChars(vec!['w'].to_letter_bitmask()),
                ),
                (1, PlaceConstraint::IsChar('e')),
                (3, PlaceConstraint::IsChar('e')),
            ]),
        };

        assert!(Word::from("rewed").matches(&pattern));
        assert!(Word::from("beweded").matches(&pattern));
        assert!(!Word::from("zeweded").matches(&pattern));

        let pattern = Pattern {
            disallowed: LetterBitmask::compute_bitmask(
                vec!['t', 'b', 'i', 'n', 'g', 's', 'z', 'e', 'l', 'u', 'y', 'r'].iter(),
            ),
            must_contain: HashMap::from_iter(vec![('a', 1), ('o', 1), ('h', 1), ('c', 1)]),
            constraints: HashMap::from_iter(vec![
                (
                    0,
                    PlaceConstraint::IsNotChars(vec!['s', 'l', 'b', 'a'].to_letter_bitmask()),
                ),
                (1, PlaceConstraint::IsChar('o')),
                (
                    2,
                    PlaceConstraint::IsNotChars(vec!['a', 'n', 't', 'y'].to_letter_bitmask()),
                ),
                (
                    3,
                    PlaceConstraint::IsNotChars(vec!['r', 'a', 'o', 'g'].to_letter_bitmask()),
                ),
                (
                    4,
                    PlaceConstraint::IsNotChars(vec!['e', 'c', 'h', 'y'].to_letter_bitmask()),
                ),
            ]),
        };

        assert!(Word::from("mocha").matches(&pattern));
        assert!(!Word::from("azygy").matches(&pattern));
        assert!(!Word::from("bocha").matches(&pattern));
    }

    #[test]
    fn test_outcome_of_guess() {
        let word = Word::from("abccdeefxr");
        let guess = Arc::new(Word::from("azdcccferr"));
        let outcome = word.outcome_of_guess(guess);
        assert_eq!(
            outcome,
            vec![
                TileOutcome::Green,
                TileOutcome::Gray,
                TileOutcome::Yellow,
                TileOutcome::Green,
                TileOutcome::Yellow,
                TileOutcome::Gray,
                TileOutcome::Yellow,
                TileOutcome::Yellow,
                TileOutcome::Gray,
                TileOutcome::Green,
            ]
        );
    }
}
