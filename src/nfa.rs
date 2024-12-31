use std::collections::{HashMap, HashSet};

use crate::parser::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    id: usize,
    // if None, epsilon transition
    transitions: HashMap<Option<char>, HashSet<usize>>,
    is_accept: bool,
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

#[derive(Debug)]
pub struct IDGenerator {
    index: usize,
}

impl IDGenerator {
    fn new() -> Self {
        Self { index: 0 }
    }

    fn next(&mut self) -> usize {
        let result = self.index;
        self.index += 1;
        result
    }
}

fn generate_state(id_generator: &mut IDGenerator, is_accept: bool) -> State {
    let id = id_generator.next();
    State::new(id, HashMap::new(), is_accept)
}

pub fn build_nfa(node: Node) -> Result<NFA, String> {
    let mut id_generator = IDGenerator::new();

    let mut start = generate_state(&mut id_generator, false);
    let start_id = start.id;

    let states = match node {
        Node::Literal(c) => {
            let mut states = build_literal(&mut id_generator, &mut start, c)?;
            states.push(start);
            states
        }
        _ => {
            return Err(format!("Unsupported node: {:?}", node));
        }
    };

    Ok(NFA {
        start_id: start_id,
        states: build_states(states),
    })
}

fn build_literal(
    id_generator: &mut IDGenerator,
    start: &mut State,
    c: char,
) -> Result<Vec<State>, String> {
    let q0 = generate_state(id_generator, true);

    start.transitions.insert(Some(c), HashSet::from([q0.id]));

    Ok(vec![q0])
}

fn build_states(states: Vec<State>) -> HashMap<usize, State> {
    let mut map = HashMap::new();
    for state in states {
        map.insert(state.id, state);
    }
    map
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
    println!("--- _match_nfa ---");
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

        let next_states: HashSet<usize> = _next_states
            .unwrap_or(&HashSet::new())
            .union(&closure)
            .cloned()
            .collect();

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
    let mut visited = HashSet::new();
    _epsilon_closure(nfa, current_state_id, &mut visited)?;
    Ok(visited)
}

fn _epsilon_closure(
    nfa: &NFA,
    current_state_id: usize,
    visited: &mut HashSet<usize>,
) -> Result<(), String> {
    let current_state = nfa.states.get(&current_state_id).unwrap();
    let binding = HashSet::new();
    let epsilon_states = current_state.transitions.get(&None).unwrap_or(&binding);
    for next_state_id in epsilon_states {
        if visited.contains(&next_state_id) {
            continue;
        }
        visited.insert(next_state_id.clone());
        _epsilon_closure(nfa, next_state_id.clone(), visited)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epsilon_closure() {
        let start_id = 0;

        // 0 -> 1 -> 2 -> 3
        let q0 = State::new(start_id, HashMap::from([(None, HashSet::from([1]))]), false);
        let q1 = State::new(1, HashMap::from([(Some('a'), HashSet::from([2]))]), false);
        let q2 = State::new(2, HashMap::from([(Some('b'), HashSet::from([3]))]), false);
        let q3 = State::new(3, HashMap::new(), true);
        let states = build_states(vec![q0, q1, q2, q3]);

        let nfa = NFA { start_id, states };
        let result = epsilon_closure(&nfa, 0);
        assert_eq!(result, Ok(HashSet::from([1])));

        //      <---
        //      |  |
        // 0 -> 1 ->-> 2 -> 3
        let q0 = State::new(start_id, HashMap::from([(None, HashSet::from([1]))]), false);
        let q1 = State::new(
            1,
            HashMap::from([(Some('a'), HashSet::from([2])), (None, HashSet::from([1]))]),
            false,
        );
        let q2 = State::new(2, HashMap::from([(Some('b'), HashSet::from([3]))]), false);
        let q3 = State::new(3, HashMap::new(), true);
        let states = build_states(vec![q0, q1, q2, q3]);
        let nfa = NFA { start_id, states };

        let result = epsilon_closure(&nfa, 0);
        assert_eq!(result, Ok(HashSet::from([1])));

        let result = epsilon_closure(&nfa, 1);
        assert_eq!(result, Ok(HashSet::from([1])));

        let result = epsilon_closure(&nfa, 2);
        assert_eq!(result, Ok(HashSet::from([])));

        let q0 = State::new(start_id, HashMap::from([(None, HashSet::from([1]))]), false);
        let q1 = State::new(
            1,
            HashMap::from([
                (Some('a'), HashSet::from([2])),
                (None, HashSet::from([0, 1])),
            ]),
            false,
        );
        let q2 = State::new(2, HashMap::from([(Some('b'), HashSet::from([3]))]), false);
        let q3 = State::new(3, HashMap::new(), true);
        let states = build_states(vec![q0, q1, q2, q3]);
        let nfa = NFA { start_id, states };

        let result = epsilon_closure(&nfa, 0);
        assert_eq!(result, Ok(HashSet::from([0, 1])));

        let result = epsilon_closure(&nfa, 1);
        assert_eq!(result, Ok(HashSet::from([0, 1])));

        let result = epsilon_closure(&nfa, 2);
        assert_eq!(result, Ok(HashSet::from([])));
    }
}
