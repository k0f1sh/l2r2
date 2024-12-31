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