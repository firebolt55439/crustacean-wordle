use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// For a specific word character position, this object records whether or not
/// the position is known to contain a specific character or known to not
/// contain a set of characters.
pub enum PlaceConstraint {
    IsChar(char),
    IsNotChars(Vec<char>),
}

/// Stores the accumulation of knowledge gained over a Wordle game, specifically
/// the characters that do not exist in the word and the positional character
/// constraints.
pub struct Pattern {
    disallowed: Vec<char>,
    constraints: HashMap<usize, PlaceConstraint>,
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
    /// is assumed to have multiple single-space-separated columns. This function
    /// requires that the first column corresponds to the word and the last column
    /// corresponds to a nonnegative score, such that higher scores indicate the
    /// word more frequently occurs.
    pub fn init(path: &str) -> Self {
        let file =
            File::open(path).expect(format!("Could not read wordlist at path '{}'", path).as_str());

        let mut words: Vec<Word> = vec![];
        let mut scores: Vec<f64> = vec![];

        let lines = BufReader::new(file).lines();

        lines.filter_map(|line| line.ok()).for_each(|line| {
            let mut columns = line.split(" ");
            let word = columns.next().expect("Could not read word from column");
            let score: f64 = columns
                .last()
                .expect("Could not read score from column")
                .parse()
                .unwrap();

            words.push(Word::from(word));
            scores.push(score);
        });

        Wordlist { words, scores }
    }
}
