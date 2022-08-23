use indicatif::ProgressBar;
use rayon::prelude::*;
use std::{collections::HashMap, sync::Arc};

use crate::{
    game::{Guess, TileOutcome},
    words::{CanPatternFilter, HasWords, Pattern, WordPtr, WordlistPtr},
};

#[derive(PartialEq, Eq)]
pub enum StrategyVerbosity {
    Silent,
    PrettyPrint,
}

/// Represents a game strategy for use with `Game`.
pub trait Strategy {
    /// All the guesses this strategy will consider making.
    fn extant_guesses(&self) -> &[WordPtr];

    /// The current best guess according to this strategy.
    fn chosen_guess(&self) -> WordPtr;

    /// A callback function for the game to register a new `Guess`
    /// with this strategy.
    fn register_guess(&mut self, guess: &Guess);

    /// Return strategy metrics.
    fn metrics(&self) -> HashMap<String, f64>;

    /// Pretty-print strategy information.
    fn pretty_print(&self, history: &Vec<HashMap<String, f64>>);

    /// Set strategy verbosity.
    fn set_verbosity(&mut self, verbosity: StrategyVerbosity);
}

pub struct EntropyStrategy {
    knowledge: Pattern,
    verbosity: StrategyVerbosity,
    extant: Arc<dyn CanPatternFilter + Send + Sync>,
}

impl Strategy for EntropyStrategy {
    fn extant_guesses(&self) -> &[WordPtr] {
        self.extant.possible_words()
    }

    fn register_guess(&mut self, guess: &Guess) {
        self.knowledge = self.knowledge.ingest(guess);
        self.extant = self.extant.filter_pattern(&self.knowledge);
    }

    fn chosen_guess(&self) -> WordPtr {
        let words = self.extant.possible_words();

        let pb = match self.verbosity {
            StrategyVerbosity::PrettyPrint => ProgressBar::new(words.len() as u64),
            _ => ProgressBar::hidden(),
        };

        let current_entropy = self.extant.unweighted_entropy();
        let best_guess = words
            .par_iter()
            // .iter()
            .map(|guess| {
                let mut possible_patterns: HashMap<Vec<TileOutcome>, usize> = HashMap::new();
                for actual_word in words {
                    let outcome = actual_word.outcome_of_guess(guess.clone());
                    *possible_patterns.entry(outcome).or_insert(0) += 1;
                }

                let mut total_gain = 0.0_f64;
                for (outcome, count) in possible_patterns {
                    let guess = Guess {
                        guess: guess.get_word().chars().collect(),
                        outcome,
                    };
                    let pattern = self.knowledge.ingest(&guess);
                    let sublist = self.extant.filter_pattern(&pattern);

                    let new_entropy = sublist.unweighted_entropy();
                    let improvement = current_entropy - new_entropy;
                    total_gain += (count as f64) * improvement;
                }

                pb.inc(1);

                // Since words.len() is constant, maximizing `total_gain` is equivalent to
                // maximizing average gain.
                (total_gain, guess)
            })
            .reduce_with(
                |(i1, g1), (i2, g2)| {
                    if i1 > i2 {
                        (i1, g1)
                    } else {
                        (i2, g2)
                    }
                },
            );

        pb.finish_and_clear();

        best_guess
            .expect("Could not compute best guess for EntropyStrategy!")
            .1
            .clone()
    }

    fn metrics(&self) -> HashMap<String, f64> {
        HashMap::from([
            (
                "extant_guesses".to_string(),
                self.extant_guesses().len() as f64,
            ),
            (
                "unweighted_entropy".to_string(),
                self.extant.unweighted_entropy(),
            ),
            (
                "weighted_entropy".to_string(),
                self.extant.weighted_entropy(),
            ),
        ])
    }

    fn pretty_print(&self, history: &Vec<HashMap<String, f64>>) {
        println!(
            "# Extant Guesses: {} (entropy: {})",
            self.extant_guesses().len(),
            self.extant.unweighted_entropy()
        );
        println!("Disallowed characters: {:?}", self.knowledge.disallowed);
        println!("Must-contain characters: {:?}", self.knowledge.must_contain);
        println!("Constraints: {:?}", self.knowledge.constraints);
        println!();

        for (idx, metrics) in history.iter().enumerate() {
            println!("History Entry #{}: {:?}", idx + 1, metrics);
        }
    }

    fn set_verbosity(&mut self, verbosity: StrategyVerbosity) {
        self.verbosity = verbosity;
    }
}

impl EntropyStrategy {
    /// Initializes a new Strategy with the given Game.
    pub fn init(wordlist: WordlistPtr) -> Box<dyn Strategy> {
        Box::new(EntropyStrategy {
            knowledge: Pattern::default(),
            verbosity: StrategyVerbosity::Silent,
            extant: wordlist.clone(),
        })
    }
}
