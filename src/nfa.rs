use std::collections::{HashMap, HashSet};

use crate::parser::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    id: usize,
    transitions: HashMap<char, HashSet<usize>>,
    is_accept: bool, // このステートが受理状態かどうか
}

impl State {
    pub fn new(id: usize, transitions: HashMap<char, HashSet<usize>>, is_accept: bool) -> Self {
        Self {
            id,
            transitions,
            is_accept,
        }
    }
}

#[derive(Debug)]
pub struct NFA {
    start_id: usize,
    states: HashMap<usize, State>,
}

impl NFA {}

pub fn build_nfa(node: Node) -> NFA {
    // TODO: This is just an example
    let start_id = 0;
    let s0 = State::new(start_id, HashMap::from([('a', HashSet::from([1]))]), false);
    let s1 = State::new(1, HashMap::from([('b', HashSet::from([2]))]), false);
    let s2 = State::new(2, HashMap::new(), true);

    // TODO: use function to build states
    let states = HashMap::from([(s0.id, s0), (s1.id, s1), (s2.id, s2)]);
    NFA { start_id, states }
}
