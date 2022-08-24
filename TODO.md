TODO:

- [x] normalize scores by z-score then log
- [x] add structure that represents a subset of the wordlist
- [x] add entropy computation helper to the wordlist subset struct and the super struct

- [x] for gameplay, add enum that represents possible guess outcomes (Gray, Yellow, Green) and `Vec<that enum>` that represents a guess
- [x] add helpers to `words::Pattern` that ingests a Guess and returns an updated Pattern
- [x] add multi-threaded strategy evaluation over wordlist by avg. number of guesses taken for top 80% of words

- [x] add Game pretty print function
- [x] add human input strategy so the user can play wordle / add a REPL
- [x] add a choose_random_word that only chooses top percentile words (done implicitly by separate answerlist)
- [x] have separate wordlists for guessing + choosing
- [x] make answer/guess list have frequency scores
- [ ] fix the height thing
- [ ] modify strategy to use weighted entropy when # of extant words is below a threshold
- [ ] evaluate strategy on all words in the set; cache initial guess
- [ ] once above is done, we can backtest improvements in the strategy
- [ ] in the strategy solver, in outer loop, consider ALL guesses (some invalid guesses may provide far more information)

# Extant Guesses: 3 (entropy: 1.584962500721156)
Disallowed characters: {'w', 's', 'r', 'a', 'n'}
Must-contain characters: {'o': 1, 'b': 1, 'd': 1, 'e': 1}
Constraints: {3: IsChar('e'), 1: IsChar('o'), 4: IsChar('d'), 2: IsNotChars(['a', 'n', 'w', 'd', 'o']), 0: IsChar('b')}


History Entry #1: {"extant_guesses": 12974.0, "unweighted_entropy": 13.663335723478353, "weighted_entropy": NaN}
History Entry #2: {"extant_guesses": 177.0, "unweighted_entropy": 7.467605550082998, "weighted_entropy": NaN}
History Entry #3: {"extant_guesses": 7.0, "unweighted_entropy": 2.807354922057604, "weighted_entropy": NaN}
History Entry #4: {"extant_guesses": 6.0, "unweighted_entropy": 2.584962500721156, "weighted_entropy": NaN}
History Entry #5: {"extant_guesses": 5.0, "unweighted_entropy": 2.321928094887362, "weighted_entropy": NaN}
History Entry #6: {"extant_guesses": 4.0, "unweighted_entropy": 2.0, "weighted_entropy": NaN}
History Entry #7: {"extant_guesses": 3.0, "unweighted_entropy": 1.584962500721156, "weighted_entropy": NaN}

+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---
Chosen word: boyed
# Guesses: 6
# Allowed Guesses: 12974 (entropy: 13.663335723478353)
+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---
Guess #1: soare [[Gray, Green, Gray, Gray, Yellow]]
Guess #2: bored [[Green, Green, Gray, Green, Green]]
Guess #3: boned [[Green, Green, Gray, Green, Green]]
Guess #4: bowed [[Green, Green, Gray, Green, Green]]
Guess #5: boded [[Green, Green, Gray, Green, Green]]
Guess #6: booed [[Green, Green, Gray, Green, Green]]
+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---
+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---
