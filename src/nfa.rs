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

    fn get_next_state(&self, c: &char) -> Option<HashSet<usize>> {
        if self.transitions.contains_key(c) {
            Some(self.transitions[&c].clone())
        } else {
            None
        }
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

    fn get_state(&self, id: &usize) -> Option<&State> {
        self.states.get(&id)
    }

    fn get_start_state(&self) -> Option<&State> {
        self.get_state(&self.start_state)
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

pub fn match_nfa(nfa: NFA, input: &str) -> bool {
    let mut it = input.chars().peekable();
    let mut current_states = HashSet::new();
    current_states.insert(nfa.start_state);

    // TODO
    // Epsilon closure
    //current_states = nfa.epsilon_closure(start_state);

    loop {
        match it.peek() {
            Some(c) => {
                let mut next_states = HashSet::new();
                for state in current_states.iter() {
                    if let Some(state) = nfa.get_state(state) {
                        // 現在の状態から遷移を探索o
                        if let Some(target_states) = state.get_next_state(c) {
                            next_states.extend(target_states);
                        }
                    }
                }
                // TODO Epsilon closure
                current_states.extend(next_states);
            }
            None => break,
        }
        it.next();
    }

    // check accept states
    println!("current_states: {:?}", current_states);
    for state in current_states {
        if nfa.accept_states.contains(&state) {
            return true;
        }
    }
    false
}

// fn epsilon_closure(nfa: NFA, state: usize) -> HashSet<usize> {
//     let mut closure = HashSet::new();
//     closure.insert(state);
//     closure
// }
