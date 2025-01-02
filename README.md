# LiteLearningRustRegex (L2R2)

LLiteLearningRustRegex (L2R2) is my personal learning project.
I started it to understand regular expressions.
Please do not use it in production environments.

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
- Alternation (`|`) - e.g. "a|b" matches "a" or "b"
- Concatenation - e.g. "ab" matches "ab"
- Grouping with parentheses (`()`) - e.g. "(a|b)c" matches "ac" or "bc"
- Quantifiers:
  - Zero or more (`*`) - e.g. "a*" matches "", "a", "aa", etc.
  - One or more (`+`) - e.g. "a+" matches "a", "aa", etc.
  - Zero or one (`?`) - e.g. "a?" matches "" or "a"
- Wildcard (`.`) - matches any single character
- Character classes (`[]`) - matches any single character in the set


## TODO

- [ ] Optimize the NFA construction
  - remove redundant epsilon transitions
- [ ] Implement `^` and `$`
- [ ] Implement repetition (e.g. `a{2,3}`)
