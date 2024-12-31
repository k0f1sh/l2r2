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

    fn add_transition(&mut self, c: Option<char>, state_id: usize) {
        self.transitions
            .entry(c)
            .or_insert(HashSet::new())
            .insert(state_id);
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
    let (states, _, _) = _build_nfa(node, &mut id_generator)?;
    Ok(NFA {
        start_id: 0,
        states: build_states(states),
    })
}

fn _build_nfa(
    node: Node,
    id_generator: &mut IDGenerator,
) -> Result<(Vec<State>, usize, usize), String> {
    let mut start = generate_state(id_generator, false);
    let mut states = vec![];
    let end_id;
    match node {
        Node::Literal(c) => {
            let (added_states, _, _end_id) = build_literal(id_generator, &mut start, c)?;
            states.extend(added_states);
            end_id = _end_id;
        }
        Node::Or(left, right) => {
            let (added_states, _, _end_id) = build_or(id_generator, &mut start, left, right)?;
            states.extend(added_states);
            end_id = _end_id;
        }
        Node::Concat(nodes) => {
            let (added_states, _, _end_id) = build_concat(id_generator, &mut start, nodes)?;
            states.extend(added_states);
            end_id = _end_id;
        }
        _ => {
            return Err(format!("Unsupported node: {:?}", node));
        }
    };

    let start_id = start.id;
    states.push(start);
    Ok((states, start_id, end_id))
}

fn build_literal(
    id_generator: &mut IDGenerator,
    start: &mut State,
    c: char,
) -> Result<(Vec<State>, usize, usize), String> {
    let q0 = generate_state(id_generator, true);
    let q0_id = q0.id;

    start.add_transition(Some(c), q0_id);

    Ok((vec![q0], q0_id, q0_id))
}

fn build_or(
    id_generator: &mut IDGenerator,
    start: &mut State,
    left: Box<Node>,
    right: Box<Node>,
) -> Result<(Vec<State>, usize, usize), String> {
    let mut q0 = generate_state(id_generator, false);
    let left = *left;
    let right = *right;
    let (mut left_states, left_start_id, left_end_id) = _build_nfa(left, id_generator)?;
    let (mut right_states, right_start_id, right_end_id) = _build_nfa(right, id_generator)?;

    // start -> q0
    start.add_transition(None, q0.id);

    // q0 -> left_states or right_states
    q0.add_transition(None, left_start_id);
    q0.add_transition(None, right_start_id);

    // end -> left_end_id or right_end_id
    let end = generate_state(id_generator, true);
    let end_id = end.id;

    // change is_accept to false
    let left_end_state = left_states
        .iter_mut()
        .find(|state| state.id == left_end_id)
        .unwrap();
    left_end_state.is_accept = false;
    left_end_state.add_transition(None, end.id);
    let right_end_state = right_states
        .iter_mut()
        .find(|state| state.id == right_end_id)
        .unwrap();
    right_end_state.is_accept = false;
    right_end_state.add_transition(None, end.id);

    let start_id = q0.id;
    // return start, left_states, right_states
    let mut states = vec![q0, end];
    states.extend(left_states.into_iter());
    states.extend(right_states.into_iter());

    Ok((states, start_id, end_id))
}

fn build_concat(
    id_generator: &mut IDGenerator,
    start: &mut State,
    nodes: Vec<Node>,
) -> Result<(Vec<State>, usize, usize), String> {
    let q0 = generate_state(id_generator, false);
    let start_id = q0.id;

    start.add_transition(None, q0.id);

    let mut prev_end_id = q0.id;
    let mut states = vec![q0];
    for node in nodes {
        let (mut added_states, _first_id, _end_id) = _build_nfa(node, id_generator)?;

        // add transition to first state
        let first_state = added_states
            .iter()
            .find(|state| state.id == _first_id)
            .unwrap();

        let prev_end_state = states
            .iter_mut()
            .find(|state| state.id == prev_end_id)
            .unwrap();
        prev_end_state.add_transition(None, first_state.id);

        let end_state = added_states
            .iter_mut()
            .find(|state| state.id == _end_id)
            .unwrap();
        prev_end_id = end_state.id;

        // if end_state is accept, change is_accept to false
        end_state.is_accept = false;

        states.extend(added_states);
    }

    // last end_state is accept
    let last_end_state = states
        .iter_mut()
        .find(|state| state.id == prev_end_id)
        .unwrap();
    last_end_state.is_accept = true;

    Ok((states, start_id, prev_end_id))
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

#[derive(Debug)]
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
        self.peek().is_none()
    }
}

fn _match_nfa(
    nfa: &NFA,
    current_state_id: usize,
    input: &mut InputWithIndex,
) -> Result<MatchResult, String> {
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

        // (state_id, is_epsilon)
        let next_states = next_states
            .into_iter()
            .map(|state_id| (state_id, closure.contains(&state_id)))
            .collect::<Vec<(usize, bool)>>();

        if next_states.is_empty() {
            // if no transition, skip current char
            input.next();
            return _match_nfa(nfa, current_state_id, input);
        }

        for (next_state_id, is_epsilon) in next_states {
            let next_state = nfa.states.get(&next_state_id).unwrap();
            // check state is accept
            if next_state.is_accept {
                return Ok(MatchResult::Match);
            } else {
                // if not accept, try next state
                let current_index = input.index;
                if !is_epsilon {
                    input.next();
                }
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

    #[test]
    fn test_match_nfa() {
        // a
        let nfa = build_nfa(Node::Literal('a')).unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "b"), Ok(false));
        assert_eq!(match_nfa(&nfa, "aa"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ab"), Ok(true));

        // ab
        let nfa = build_nfa(Node::Concat(vec![Node::Literal('a'), Node::Literal('b')])).unwrap();
        assert_eq!(match_nfa(&nfa, "ab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "aab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ba"), Ok(false));
        assert_eq!(match_nfa(&nfa, "a"), Ok(false));
        assert_eq!(match_nfa(&nfa, "b"), Ok(false));

        // a|b
        let nfa = build_nfa(Node::Or(
            Box::new(Node::Literal('a')),
            Box::new(Node::Literal('b')),
        ))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "b"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ba"), Ok(true));
        assert_eq!(match_nfa(&nfa, "bb"), Ok(true));

        // ab|cd
        let nfa = build_nfa(Node::Or(
            Box::new(Node::Concat(vec![Node::Literal('a'), Node::Literal('b')])),
            Box::new(Node::Concat(vec![Node::Literal('c'), Node::Literal('d')])),
        ))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "ab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "cd"), Ok(true));
        assert_eq!(match_nfa(&nfa, "abcd"), Ok(true));
        assert_eq!(match_nfa(&nfa, "abd"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ac"), Ok(false));
        assert_eq!(match_nfa(&nfa, "ad"), Ok(false));
        assert_eq!(match_nfa(&nfa, "bc"), Ok(false));
        assert_eq!(match_nfa(&nfa, "bd"), Ok(false));
        assert_eq!(match_nfa(&nfa, "abc"), Ok(true));
        assert_eq!(match_nfa(&nfa, "abd"), Ok(true));
        assert_eq!(match_nfa(&nfa, "acd"), Ok(true));
        assert_eq!(match_nfa(&nfa, "bcd"), Ok(true));
    }
}
