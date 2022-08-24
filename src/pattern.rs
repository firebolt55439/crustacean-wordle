use crate::bitmask::*;
use crate::game::{Guess, TileOutcome};
use std::collections::HashMap;

/// For a specific word character position, this object records whether or not
/// the position is known to contain a specific character or known to not
/// contain a set of characters.
#[derive(Debug, Clone, PartialEq)]
pub enum PlaceConstraint {
    IsChar(char),
    IsNotChars(LetterBitmask),
}

/// Stores the accumulation of knowledge gained over a Wordle game, specifically
/// the characters that do not exist in the word and the positional character
/// constraints.
#[derive(Default)]
pub struct Pattern {
    pub disallowed: LetterBitmask,
    pub must_contain: HashMap<char, usize>,
    pub constraints: HashMap<usize, PlaceConstraint>,
}

impl Pattern {
    /// Update the knowledge contained within this `Pattern` with the outcome
    /// of `guess`.
    pub fn ingest(&self, guess: &Guess) -> Self {
        let mut disallowed = self.disallowed.clone();
        let mut must_contain = self.must_contain.clone();

        let mut constraints: HashMap<usize, PlaceConstraint> = HashMap::new();
        constraints.reserve(self.constraints.len());
        let constraints_kvs = self
            .constraints
            .iter()
            .map(|(a, b)| (a.to_owned(), b.to_owned()));
        constraints.extend(constraints_kvs);

        let mut count_map: HashMap<&char, usize> = HashMap::new(); // store the min count of characters that appear in yellow at least once
        let mut to_disallow: Vec<&char> = Vec::new(); // what we will consider banning entirely (depends on if seen in string)

        for (idx, (ch, outcome)) in guess.paired_iter().enumerate() {
            let mask = LetterBitmask::char_bitmask(ch);
            match outcome {
                TileOutcome::Gray | TileOutcome::Yellow => {
                    if outcome == &TileOutcome::Gray {
                        to_disallow.push(ch);
                    } else {
                        *count_map.entry(ch).or_insert(0) += 1;
                    }

                    if disallowed & mask == 0 {
                        if let Some(cons) = constraints.get_mut(&idx) {
                            match cons {
                                PlaceConstraint::IsChar(_) => {}
                                PlaceConstraint::IsNotChars(chars) => {
                                    *chars |= mask;
                                }
                            }
                        } else {
                            constraints.insert(idx, PlaceConstraint::IsNotChars(mask));
                        }
                    }
                }
                TileOutcome::Green => {
                    *count_map.entry(ch).or_insert(0) += 1;
                    constraints.insert(idx, PlaceConstraint::IsChar(ch.to_owned()));
                }
            }
        }

        for (ch, count) in count_map.iter() {
            let entry = must_contain.entry(**ch).or_insert(*count);
            *entry = (*entry).max(*count);
        }

        for ch in to_disallow {
            let mask = LetterBitmask::char_bitmask(ch);
            if !count_map.contains_key(ch) && disallowed & mask == 0 {
                disallowed |= mask;
            }
        }

        Pattern {
            disallowed,
            must_contain,
            constraints,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest() {
        let pattern = Pattern::default();
        let pattern1 = pattern.ingest(&Guess {
            guess: vec!['t', 'a', 'r', 'e', 's'],
            outcome: vec![
                TileOutcome::Yellow,
                TileOutcome::Gray,
                TileOutcome::Gray,
                TileOutcome::Gray,
                TileOutcome::Gray,
            ],
        });

        assert_eq!(
            pattern1.disallowed,
            LetterBitmask::compute_bitmask(vec!['r', 'a', 's', 'e'].iter())
        );
        assert_eq!(
            pattern1.must_contain,
            HashMap::from_iter(vec![('t', 1_usize)])
        );
        assert_eq!(
            pattern1.constraints,
            HashMap::from_iter(vec![
                (
                    0,
                    PlaceConstraint::IsNotChars(vec!['t'].to_letter_bitmask())
                ),
                (
                    1,
                    PlaceConstraint::IsNotChars(vec!['a'].to_letter_bitmask())
                ),
                (
                    2,
                    PlaceConstraint::IsNotChars(vec!['r'].to_letter_bitmask())
                ),
                (
                    3,
                    PlaceConstraint::IsNotChars(vec!['e'].to_letter_bitmask())
                ),
                (
                    4,
                    PlaceConstraint::IsNotChars(vec!['s'].to_letter_bitmask())
                ),
            ])
        );

        let pattern = Pattern {
            disallowed: LetterBitmask::compute_bitmask(vec!['a', 'b', 'c'].iter()),
            must_contain: HashMap::from_iter(vec![('e', 1_usize)]),
            constraints: HashMap::from_iter(vec![
                (0, PlaceConstraint::IsChar('d')),
                (1, PlaceConstraint::IsChar('e')),
                (2, PlaceConstraint::IsChar('d')),
            ]),
        };

        let pattern2 = pattern.ingest(&Guess {
            guess: vec!['d', 'e', 'd', 'e', 'f', 'e'],
            outcome: vec![
                TileOutcome::Green,
                TileOutcome::Green,
                TileOutcome::Green,
                TileOutcome::Yellow,
                TileOutcome::Yellow,
                TileOutcome::Gray,
            ],
        });

        assert_eq!(
            pattern2.disallowed,
            LetterBitmask::compute_bitmask(vec!['a', 'b', 'c'].iter())
        );
        assert_eq!(
            pattern2.must_contain,
            HashMap::from_iter(vec![('f', 1_usize), ('d', 2_usize), ('e', 2_usize)])
        );
        assert_eq!(
            pattern2.constraints,
            HashMap::from_iter(vec![
                (0, PlaceConstraint::IsChar('d')),
                (1, PlaceConstraint::IsChar('e')),
                (2, PlaceConstraint::IsChar('d')),
                (
                    3,
                    PlaceConstraint::IsNotChars(vec!['e'].to_letter_bitmask())
                ),
                (
                    4,
                    PlaceConstraint::IsNotChars(vec!['f'].to_letter_bitmask())
                ),
                (
                    5,
                    PlaceConstraint::IsNotChars(vec!['e'].to_letter_bitmask())
                ),
            ])
        );
    }
}
