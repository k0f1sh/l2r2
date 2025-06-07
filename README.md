# LiteLearningRustRegex (L2R2)

LLiteLearningRustRegex (L2R2) is my personal learning project.
I started it to understand regular expressions.
Please do not use it in production environments.

## Usage (grep)

### NFA-based grep (default)
```bash
echo "a" | cargo run "a|b"
```

### DFA-based grep (faster)
```bash
echo "a" | cargo run --bin dfa_grep "a|b"
```

## Usage (visualization)

### NFA visualization
For debugging purposes, you can visualize the NFA (Non-deterministic Finite Automaton) using Graphviz:

```bash
cargo run --bin dot "a|b" > nfa.dot
dot -Tpng nfa.dot -o nfa.png
```

### DFA visualization
You can also visualize the DFA (Deterministic Finite Automaton) converted from NFA:

```bash
cargo run --bin dfa_dot "a|b" > dfa.dot
dot -Tpng dfa.dot -o dfa.png
```

## Features

### Regex Syntax Support
Currently supported regex syntax:

- Basic characters (e.g. "a", "b", "c")
- Alternation (`|`) - e.g. "a|b" matches "a" or "b"
- Concatenation - e.g. "ab" matches "ab"
- Grouping with parentheses (`()`) - e.g. "(a|b)c" matches "ac" or "bc"
- Quantifiers:
  - Zero or more (`*`) - e.g. "a*" matches "", "a", "aa", etc.
  - One or more (`+`) - e.g. "a+" matches "a", "aa", etc.
  - Zero or one (`?`) - e.g. "a?" matches "" or "a"
- Wildcard (`.`) - matches any single character
- Character classes (`[]`) - matches any single character in the set
- `^` and `$` - matches the start and end of the string

### Engine Types
- **NFA Engine**: Non-deterministic finite automaton with epsilon transitions
- **DFA Engine**: Deterministic finite automaton converted from NFA using subset construction
  - Faster matching performance (no backtracking)
  - Simpler state transitions (no epsilon moves)
  - Larger memory footprint for complex patterns

## TODO

- [x] ~~Optimize the NFA construction~~ ✅ **DONE**: Added DFA conversion for better performance
- [ ] Implement repetition (e.g. `a{2,3}`)
- [ ] Implement character classes (e.g. `\S` `\d` `\w` `\s`)
- [ ] Add performance benchmarks comparing NFA vs DFA
- [ ] Implement DFA minimization algorithm
