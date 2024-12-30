use std::{
    collections::{HashMap, HashSet},
    iter::Peekable,
};

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
    let q0 = State::new(start_id, HashMap::from([('a', HashSet::from([1]))]), false);
    let q1 = State::new(1, HashMap::from([('b', HashSet::from([2]))]), false);
    let q2 = State::new(2, HashMap::new(), true);

    // TODO: use function to build states
    let states = HashMap::from([(q0.id, q0), (q1.id, q1), (q2.id, q2)]);
    NFA { start_id, states }
}

pub fn match_nfa(nfa: &NFA, input: &str) -> Result<bool, String> {
    let result = _match_nfa(
        nfa,
        nfa.start_id,
        &mut InputWithIndex {
            index: 0,
            input: input.to_string(),
        },
    )?;
    match result {
        MatchResult::Match => Ok(true),
        MatchResult::NoMatch => Ok(false),
    }
}

// TODO: boolでいいかも
enum MatchResult {
    Match,
    NoMatch,
}

struct InputWithIndex {
    index: usize,
    input: String,
}

impl InputWithIndex {
    fn next(&mut self) -> Option<char> {
        let result = self.input.chars().nth(self.index);
        self.index += 1;
        result
    }
    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.index)
    }
    fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

fn _match_nfa(
    nfa: &NFA,
    current_state_id: usize,
    input: &mut InputWithIndex,
) -> Result<MatchResult, String> {
    if let Some(c) = input.peek() {
        // TODO: epsilon transition
        //
        println!("current_state_id: {}", current_state_id);
        println!("current_char: {}", c);

        // check transition
        let next_states = nfa
            .states
            .get(&current_state_id)
            .and_then(|state| state.transitions.get(&c));

        if next_states.is_none() {
            // if no transition, skip current char
            input.next();
            return _match_nfa(nfa, current_state_id, input);
        }

        for next_state_id in next_states.unwrap() {
            let next_state = nfa.states.get(&next_state_id).unwrap();
            // check state is accept
            if next_state.is_accept {
                return Ok(MatchResult::Match);
            } else {
                // if not accept, try next state
                let current_index = input.index;
                input.next();
                let result = _match_nfa(nfa, next_state_id.clone(), input)?;
                match result {
                    MatchResult::Match => return Ok(MatchResult::Match),
                    MatchResult::NoMatch => {
                        // if no match, reset index
                        input.set_index(current_index);
                        continue;
                    }
                }
            }
        }
    }
    Ok(MatchResult::NoMatch)
}
