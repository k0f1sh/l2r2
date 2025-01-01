# LiteLearningRustRegex (L2R2)

LiteLearningRustRegex (L2R2) is a lightweight project designed for exploring and understanding how regex engines work.
This project is a learning-focused.

## Usage (grep)

```bash
echo "a" | cargo run "a|b"
```

## Usage (dot)

For debugging purposes, you can visualize the NFA (Non-deterministic Finite Automaton) using Graphviz. The `dot` binary generates a DOT format representation of the NFA, which can then be converted to an image using the `dot` command-line tool:


```bash
cargo run --bin dot "a|b" > nfa.dot
dot -Tpng nfa.dot -o nfa.png
```

## Features

Currently supported regex syntax: 

- Basic characters (e.g. "a", "b", "c")
- Alternation (|) - e.g. "a|b" matches "a" or "b"
- Concatenation - e.g. "ab" matches "ab"
- Grouping with parentheses - e.g. "(a|b)c" matches "ac" or "bc"
- Quantifiers:
  - Zero or more (*) - e.g. "a*" matches "", "a", "aa", etc.
  - One or more (+) - e.g. "a+" matches "a", "aa", etc.
  - Zero or one (?) - e.g. "a?" matches "" or "a"
- Wildcard (.) - matches any single character
- Character classes ([abc]) - matches any single character in the set


## TODO

- [ ] Optimize the NFA construction
  - remove redundant epsilon transitions
- [ ] Implement `^` and `$`
- [ ] Implement repetition (e.g. `a{2,3}`)
