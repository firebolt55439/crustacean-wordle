TODO:

- [x] normalize scores by z-score then log
- [x] add structure that represents a subset of the wordlist
- [x] add entropy computation helper to the wordlist subset struct and the super struct

- [x] for gameplay, add enum that represents possible guess outcomes (Gray, Yellow, Green) and `Vec<that enum>` that represents a guess
- [x] add helpers to `words::Pattern` that ingests a Guess and returns an updated Pattern
- [x] add multi-threaded strategy evaluation over wordlist by avg. number of guesses taken for top 80% of words

- [ ] add Game pretty print function
- [ ] modify strategy to use weighted entropy when # of extant words is below a threshold
- [ ] add human input strategy so the user can play wordle / add a REPL
- [ ] evaluate strategy on all words in the set; cache initial guess
- [ ] once above is done, we can backtest improvements in the strategy
