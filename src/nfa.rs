use std::collections::{HashMap, HashSet};

use crate::parser::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    id: usize,
    transitions: HashMap<char, HashSet<usize>>,
}

impl State {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            transitions: HashMap::new(),
        }
    }

    fn add_transition(&mut self, c: char, to: usize) {
        self.transitions
            .entry(c)
            .or_insert_with(HashSet::new)
            .insert(to);
    }
}

#[derive(Debug)]
pub struct NFA {
    start_state: usize,
    accept_states: HashSet<usize>,
    states: HashMap<usize, State>,
}

impl NFA {
    pub fn new() -> Self {
        Self {
            start_state: 0,
            accept_states: HashSet::new(),
            states: HashMap::new(),
        }
    }

    fn add_state(&mut self, state: State) {
        self.states.insert(state.id, state);
    }

    fn add_accept_state(&mut self, state: usize) {
        self.accept_states.insert(state);
    }
}

pub fn build_nfa(node: Node) -> Result<NFA, String> {
    let mut nfa = NFA::new();
    let mut current_id = 0;

    loop {
        match node {
            Node::Literal(c) => {
                let mut state = State::new(current_id);

                state.add_transition(c, current_id + 1);
                nfa.add_state(state);
                nfa.add_accept_state(current_id + 1);

                // Add end state
                current_id += 1;
                let end_state = State::new(current_id);
                nfa.add_state(end_state);
                nfa.add_accept_state(current_id);
                break;
            }
            _ => return Err("Unsupported node type".to_string()),
        }
    }

    Ok(nfa)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_nfa() {}
}
