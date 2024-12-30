use std::collections::{HashMap, HashSet};

use crate::parser::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    id: usize,
    // if None, epsilon transition
    transitions: HashMap<Option<char>, HashSet<usize>>,
    is_accept: bool, // このステートが受理状態かどうか
}

impl State {
    pub fn new(
        id: usize,
        transitions: HashMap<Option<char>, HashSet<usize>>,
        is_accept: bool,
    ) -> Self {
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

pub fn build_nfa(node: Node) -> NFA {
    // TODO: This is just an example
    let start_id = 0;
    let q0 = State::new(start_id, HashMap::from([(None, HashSet::from([1]))]), false);
    let q1 = State::new(1, HashMap::from([(Some('a'), HashSet::from([2]))]), false);
    let q2 = State::new(2, HashMap::from([(Some('b'), HashSet::from([3]))]), false);
    let q3 = State::new(3, HashMap::new(), true);

    // TODO: use function to build states
    let states = HashMap::from([(q0.id, q0), (q1.id, q1), (q2.id, q2), (q3.id, q3)]);
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

#[derive(Debug)]
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
    fn is_end(&self) -> bool {
        self.index >= self.input.len()
    }
}

fn _match_nfa(
    nfa: &NFA,
    current_state_id: usize,
    input: &mut InputWithIndex,
) -> Result<MatchResult, String> {
    println!("input: {:#?}", input);
    println!("current_state_id: {}", current_state_id);
    if input.is_end() {
        let closure = epsilon_closure(nfa, current_state_id)?;
        for state_id in closure {
            if nfa.states.get(&state_id).unwrap().is_accept {
                return Ok(MatchResult::Match);
            }
        }
        return Ok(MatchResult::NoMatch);
    }

    if let Some(c) = input.peek() {
        // check transition
        let _next_states = nfa
            .states
            .get(&current_state_id)
            .and_then(|state| state.transitions.get(&Some(c)));

        // check epsilon transition
        let closure = epsilon_closure(nfa, current_state_id)?;

        let mut next_states: HashSet<usize> = _next_states
            .unwrap_or(&HashSet::new())
            .union(&closure)
            .cloned()
            .collect();
        // FIXME: Is this correct?
        // remove current_state_id (for epsilon transition)
        next_states.remove(&current_state_id);

        if next_states.is_empty() {
            // if no transition, skip current char
            input.next();
            return _match_nfa(nfa, current_state_id, input);
        }

        for next_state_id in next_states {
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

fn epsilon_closure(nfa: &NFA, current_state_id: usize) -> Result<HashSet<usize>, String> {
    _epsilon_closure(nfa, current_state_id, &mut HashSet::new())
}

fn _epsilon_closure(
    nfa: &NFA,
    current_state_id: usize,
    visited: &mut HashSet<usize>,
) -> Result<HashSet<usize>, String> {
    visited.insert(current_state_id);
    let mut closure = HashSet::from([current_state_id]);

    let current_state = nfa.states.get(&current_state_id).unwrap();
    let binding = HashSet::new();
    let epsilon_states = current_state.transitions.get(&None).unwrap_or(&binding);
    for next_state_id in epsilon_states {
        if visited.contains(&next_state_id) {
            continue;
        }
        let result = _epsilon_closure(nfa, next_state_id.clone(), visited)?;
        closure.extend(result);
    }
    Ok(closure)
}
