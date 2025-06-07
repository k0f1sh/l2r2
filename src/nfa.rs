#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use crate::parser::Node;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransitionKey {
    Epsilon,
    Literal(char),
    CharClass(Vec<char>),
    AnyChar,
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub id: usize,
    pub transitions: HashMap<TransitionKey, HashSet<usize>>,
    pub is_accept: bool,
}

impl State {
    pub fn new(
        id: usize,
        transitions: HashMap<TransitionKey, HashSet<usize>>,
        is_accept: bool,
    ) -> Self {
        Self {
            id,
            transitions,
            is_accept,
        }
    }

    fn add_transition(&mut self, key: TransitionKey, state_id: usize) {
        self.transitions
            .entry(key)
            .or_default()
            .insert(state_id);
    }

    #[allow(dead_code)]
    fn is_only_one_epsilon_transition(&self) -> bool {
        self.transitions.len() == 1
            && self.transitions.contains_key(&TransitionKey::Epsilon)
            && self.transitions.get(&TransitionKey::Epsilon).unwrap().len() == 1
    }

    #[allow(dead_code)]
    fn get_if_only_one_epsilon_transition(&self) -> Option<usize> {
        if self.is_only_one_epsilon_transition() {
            Some(
                *self.transitions
                    .get(&TransitionKey::Epsilon)
                    .unwrap()
                    .iter()
                    .next()
                    .unwrap(),
            )
        } else {
            None
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct NFA {
    pub start_id: usize,
    pub states: HashMap<usize, State>,
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
    let (mut states, _, end_id) = match node {
        Node::Literal(c) => build_literal(id_generator, &mut start, c)?,
        Node::Or(left, right) => build_or(id_generator, &mut start, left, right)?,
        Node::Concat(nodes) => build_concat(id_generator, &mut start, nodes)?,
        Node::ZeroOrMore(node) => build_zero_or_more(id_generator, &mut start, node)?,
        Node::OneOrMore(node) => build_one_or_more(id_generator, &mut start, node)?,
        Node::ZeroOrOne(node) => build_zero_or_one(id_generator, &mut start, node)?,
        Node::Group(node) => build_group(id_generator, &mut start, node)?,
        Node::AnyChar => build_any_char(id_generator, &mut start)?,
        Node::CharClass(chars) => build_char_class(id_generator, &mut start, chars)?,
        Node::Start => build_start(id_generator, &mut start)?,
        Node::End => build_end(id_generator, &mut start)?,
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

    start.add_transition(TransitionKey::Literal(c), q0_id);

    Ok((vec![q0], q0_id, q0_id))
}

fn build_or(
    id_generator: &mut IDGenerator,
    start: &mut State,
    left: Box<Node>,
    right: Box<Node>,
) -> Result<(Vec<State>, usize, usize), String> {
    let left = *left;
    let right = *right;
    let (mut left_states, left_start_id, left_end_id) = _build_nfa(left, id_generator)?;
    let (mut right_states, right_start_id, right_end_id) = _build_nfa(right, id_generator)?;

    // start -> left_states or right_states
    start.add_transition(TransitionKey::Epsilon, left_start_id);
    start.add_transition(TransitionKey::Epsilon, right_start_id);

    // end -> left_end_id or right_end_id
    let end = generate_state(id_generator, true);
    let end_id = end.id;

    // change is_accept to false
    let left_end_state = left_states
        .iter_mut()
        .find(|state| state.id == left_end_id)
        .unwrap();
    left_end_state.is_accept = false;
    left_end_state.add_transition(TransitionKey::Epsilon, end.id);
    let right_end_state = right_states
        .iter_mut()
        .find(|state| state.id == right_end_id)
        .unwrap();
    right_end_state.is_accept = false;
    right_end_state.add_transition(TransitionKey::Epsilon, end.id);

    // return start, left_states, right_states
    let mut states = vec![end];
    states.extend(left_states);
    states.extend(right_states);

    Ok((states, start.id, end_id))
}

// Remove duplicate transitions that point to the same target with the same key
fn remove_duplicate_transitions(states: &mut Vec<State>, start: &mut State) -> Result<(), String> {
    let mut total_removals = 0;

    // Clean up start state transitions
    for (_, target_set) in start.transitions.iter_mut() {
        let original_len = target_set.len();
        // HashSet automatically removes duplicates, but let's count them
        total_removals += original_len - target_set.len();
    }

    // Clean up all state transitions
    for state in states.iter_mut() {
        for (_, target_set) in state.transitions.iter_mut() {
            let original_len = target_set.len();
            // HashSet should handle duplicates, but let's also remove redundant states
            total_removals += original_len - target_set.len();
        }
    }

    #[cfg(debug_assertions)]
    if total_removals > 0 {
        eprintln!("DEBUG: Removed {} duplicate transitions", total_removals);
    }

    Ok(())
}

// Remove redundant intermediate states that only serve as pass-through
fn remove_redundant_states(states: &mut Vec<State>, start: &mut State) -> Result<(), String> {
    let mut removed_states = 0;

    // Find states that can be bypassed
    let mut bypass_candidates = Vec::new();

    for state in states.iter() {
        // If a state is not accept and all its outgoing transitions go to the same target
        // with the same key as incoming transitions, it might be redundant
        if !state.is_accept {
            // Check if this state has incoming transitions from multiple sources
            // that could be directly connected to its target
            let mut targets_by_key: HashMap<TransitionKey, HashSet<usize>> = HashMap::new();

            for (key, target_set) in &state.transitions {
                for target in target_set {
                    targets_by_key
                        .entry(key.clone())
                        .or_default()
                        .insert(*target);
                }
            }

            // If this state has exactly one outgoing transition type
            if targets_by_key.len() == 1 {
                let (out_key, out_targets) = targets_by_key.iter().next().unwrap();
                if out_targets.len() == 1 {
                    let target_id = *out_targets.iter().next().unwrap();

                    // Find all states that point to this state with the same key
                    let mut sources = Vec::new();

                    // Check start state
                    for (in_key, source_set) in &start.transitions {
                        if in_key == out_key && source_set.contains(&state.id) {
                            sources.push(("start", 0));
                        }
                    }

                    // Check other states
                    for source_state in states.iter() {
                        if source_state.id != state.id {
                            for (in_key, source_set) in &source_state.transitions {
                                if in_key == out_key && source_set.contains(&state.id) {
                                    sources.push(("state", source_state.id));
                                }
                            }
                        }
                    }

                    // If we found sources with the same key, this state is redundant
                    if !sources.is_empty() {
                        bypass_candidates.push((state.id, target_id, out_key.clone(), sources));
                    }
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    if !bypass_candidates.is_empty() {
        eprintln!(
            "DEBUG: Found {} redundant states to bypass",
            bypass_candidates.len()
        );
        for (state_id, target_id, key, sources) in &bypass_candidates {
            eprintln!(
                "  State {} -> {} (key: {:?}), sources: {} ",
                state_id,
                target_id,
                key,
                sources.len()
            );
        }
    }

    // Apply bypasses
    for (redundant_state_id, target_id, key, sources) in bypass_candidates {
        // Update all source states/start to point directly to target
        for (source_type, source_id) in sources {
            if source_type == "start" {
                if let Some(target_set) = start.transitions.get_mut(&key) {
                    target_set.remove(&redundant_state_id);
                    target_set.insert(target_id);
                }
            } else {
                for state in states.iter_mut() {
                    if state.id == source_id {
                        if let Some(target_set) = state.transitions.get_mut(&key) {
                            target_set.remove(&redundant_state_id);
                            target_set.insert(target_id);
                        }
                    }
                }
            }
        }
        removed_states += 1;
    }

    #[cfg(debug_assertions)]
    if removed_states > 0 {
        eprintln!("DEBUG: Bypassed {} redundant states", removed_states);
    }

    Ok(())
}

// Remove states that become unreachable after optimization
fn remove_unreachable_states(states: &mut Vec<State>, start: &mut State) -> Result<(), String> {
    // Find all reachable states
    let mut reachable = HashSet::new();
    let mut to_visit = vec![start.id];

    // Add start state as reachable
    reachable.insert(start.id);

    while let Some(current_id) = to_visit.pop() {
        // Add transitions from start state
        if current_id == start.id {
            for target_set in start.transitions.values() {
                for target_id in target_set {
                    if !reachable.contains(target_id) {
                        reachable.insert(*target_id);
                        to_visit.push(*target_id);
                    }
                }
            }
        }

        // Add transitions from regular states
        if let Some(state) = states.iter().find(|s| s.id == current_id) {
            for target_set in state.transitions.values() {
                for target_id in target_set {
                    if !reachable.contains(target_id) {
                        reachable.insert(*target_id);
                        to_visit.push(*target_id);
                    }
                }
            }
        }
    }

    let original_count = states.len();
    states.retain(|state| reachable.contains(&state.id));
    let removed_count = original_count - states.len();

    #[cfg(debug_assertions)]
    if removed_count > 0 {
        eprintln!("DEBUG: Removed {} unreachable states", removed_count);
    }

    Ok(())
}

// More targeted and safe epsilon optimization for concat chains
fn optimize_concat_epsilon_transitions(
    states: &mut Vec<State>,
    start: &mut State,
) -> Result<(), String> {
    let mut total_optimizations = 0;

    // Repeat optimization until no more improvements can be made
    loop {
        // Only optimize specific patterns: states with single epsilon transition that are not:
        // 1. Accept states
        // 2. Start/End related
        // 3. States that other states point to multiple times

        // Count incoming transitions for each state
        let mut incoming_counts: HashMap<usize, usize> = HashMap::new();
        for state in states.iter() {
            for (_, target_set) in state.transitions.iter() {
                for target_id in target_set {
                    *incoming_counts.entry(*target_id).or_insert(0) += 1;
                }
            }
        }

        // Also count from start state
        for (_, target_set) in start.transitions.iter() {
            for target_id in target_set {
                *incoming_counts.entry(*target_id).or_insert(0) += 1;
            }
        }

        // Find safe candidates for optimization
        let mut optimization_candidates = Vec::new();
        for state in states.iter() {
            if !state.is_accept {
                // Don't optimize accept states
                if let Some(target_id) = state.get_if_only_one_epsilon_transition() {
                    // Only optimize if this state has exactly one incoming transition
                    if incoming_counts.get(&state.id).unwrap_or(&0) == &1 {
                        // And the target is not a Start/End related state
                        let target_state = states.iter().find(|s| s.id == target_id).unwrap();
                        let has_start_end_transitions = target_state
                            .transitions
                            .keys()
                            .any(|key| matches!(key, TransitionKey::Start | TransitionKey::End));

                        if !has_start_end_transitions {
                            optimization_candidates.push((state.id, target_id));
                        }
                    }
                }
            }
        }

        if optimization_candidates.is_empty() {
            break; // No more optimizations possible
        }

        total_optimizations += optimization_candidates.len();

        #[cfg(debug_assertions)]
        {
            eprintln!(
                "DEBUG: Optimizing {} epsilon transitions in this round",
                optimization_candidates.len()
            );
            for (from, to) in &optimization_candidates {
                eprintln!("  {} -> {}", from, to);
            }
        }

        // Apply optimizations
        for (from_id, to_id) in optimization_candidates {
            // Find the target state's transitions
            let target_transitions = states
                .iter()
                .find(|s| s.id == to_id)
                .unwrap()
                .transitions
                .clone();
            let target_is_accept = states.iter().find(|s| s.id == to_id).unwrap().is_accept;

            // Update all states that point to from_id to point to to_id instead
            for state in states.iter_mut() {
                for (_, target_set) in state.transitions.iter_mut() {
                    if target_set.contains(&from_id) {
                        target_set.remove(&from_id);
                        target_set.insert(to_id);
                    }
                }
            }

            // Update start state if it points to from_id
            for (_, target_set) in start.transitions.iter_mut() {
                if target_set.contains(&from_id) {
                    target_set.remove(&from_id);
                    target_set.insert(to_id);
                }
            }

            // Update the from_state to have the target_state's transitions and properties
            let from_state = states.iter_mut().find(|s| s.id == from_id).unwrap();
            from_state.transitions = target_transitions;
            from_state.is_accept = target_is_accept;
        }
    }

    #[cfg(debug_assertions)]
    if total_optimizations > 0 {
        eprintln!(
            "DEBUG: Total epsilon optimizations applied: {}",
            total_optimizations
        );
    }

    // After epsilon optimization, remove duplicate transitions
    remove_duplicate_transitions(states, start)?;

    // Try to remove redundant intermediate states
    remove_redundant_states(states, start)?;

    // Remove unreachable states after optimization
    remove_unreachable_states(states, start)?;

    Ok(())
}

fn build_concat(
    id_generator: &mut IDGenerator,
    start: &mut State,
    nodes: Vec<Node>,
) -> Result<(Vec<State>, usize, usize), String> {
    let start_id = start.id;

    let mut prev_end_id = start_id;
    let mut states: Vec<State> = vec![];
    for node in nodes {
        let (mut added_states, _first_id, _end_id) = _build_nfa(node, id_generator)?;

        if prev_end_id == start_id {
            start.add_transition(TransitionKey::Epsilon, _first_id);
        } else {
            // add transition to first state
            let first_state = added_states
                .iter()
                .find(|state| state.id == _first_id)
                .unwrap();

            let prev_end_state = states
                .iter_mut()
                .find(|state| state.id == prev_end_id)
                .unwrap();
            prev_end_state.add_transition(TransitionKey::Epsilon, first_state.id);
        }

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

    // Apply more targeted epsilon optimization
    optimize_concat_epsilon_transitions(&mut states, start)?;

    Ok((states, start_id, prev_end_id))
}

fn build_zero_or_more(
    id_generator: &mut IDGenerator,
    start: &mut State,
    node: Box<Node>,
) -> Result<(Vec<State>, usize, usize), String> {
    // start is not accept
    start.is_accept = false;

    let mut end_state = generate_state(id_generator, true);
    let (mut added_states, _first_id, _end_id) = _build_nfa(*node, id_generator)?;

    // start -> first_state
    start.add_transition(TransitionKey::Epsilon, _first_id);
    start.add_transition(TransitionKey::Epsilon, end_state.id);

    // end_state -> first_state
    end_state.add_transition(TransitionKey::Epsilon, _first_id);

    // _end_state -> end_state
    let _end_state = added_states
        .iter_mut()
        .find(|state| state.id == _end_id)
        .unwrap();
    _end_state.add_transition(TransitionKey::Epsilon, end_state.id);
    _end_state.add_transition(TransitionKey::Epsilon, _first_id);
    _end_state.is_accept = false;

    let end_id = end_state.id;
    let mut states = vec![end_state];
    states.extend(added_states);

    Ok((states, start.id, end_id))
}

fn build_one_or_more(
    id_generator: &mut IDGenerator,
    start: &mut State,
    node: Box<Node>,
) -> Result<(Vec<State>, usize, usize), String> {
    let (mut added_states, _first_id, _end_id) = _build_nfa(*node, id_generator)?;

    start.add_transition(TransitionKey::Epsilon, _first_id);

    let child_end_state = added_states
        .iter_mut()
        .find(|state| state.id == _end_id)
        .unwrap();
    child_end_state.is_accept = true;
    child_end_state.add_transition(TransitionKey::Epsilon, _first_id);
    let end_id = child_end_state.id;

    Ok((added_states, start.id, end_id))
}

fn build_zero_or_one(
    id_generator: &mut IDGenerator,
    start: &mut State,
    node: Box<Node>,
) -> Result<(Vec<State>, usize, usize), String> {
    let (added_states, _first_id, _end_id) = _build_nfa(*node, id_generator)?;

    start.add_transition(TransitionKey::Epsilon, _first_id);
    start.add_transition(TransitionKey::Epsilon, _end_id);

    Ok((added_states, start.id, _end_id))
}

fn build_group(
    id_generator: &mut IDGenerator,
    start: &mut State,
    node: Box<Node>,
) -> Result<(Vec<State>, usize, usize), String> {
    let (added_states, _first_id, _end_id) = _build_nfa(*node, id_generator)?;
    start.add_transition(TransitionKey::Epsilon, _first_id);
    Ok((added_states, start.id, _end_id))
}

fn build_any_char(
    id_generator: &mut IDGenerator,
    start: &mut State,
) -> Result<(Vec<State>, usize, usize), String> {
    let q0 = generate_state(id_generator, true);
    let q0_id = q0.id;

    start.add_transition(TransitionKey::AnyChar, q0_id);

    Ok((vec![q0], q0_id, q0_id))
}

fn build_start(
    id_generator: &mut IDGenerator,
    start: &mut State,
) -> Result<(Vec<State>, usize, usize), String> {
    let q0 = generate_state(id_generator, true);
    let q0_id = q0.id;

    start.add_transition(TransitionKey::Start, q0_id);

    Ok((vec![q0], q0_id, q0_id))
}

fn build_end(
    id_generator: &mut IDGenerator,
    start: &mut State,
) -> Result<(Vec<State>, usize, usize), String> {
    let q0 = generate_state(id_generator, true);
    let q0_id = q0.id;

    start.add_transition(TransitionKey::End, q0_id);

    Ok((vec![q0], q0_id, q0_id))
}

fn build_char_class(
    id_generator: &mut IDGenerator,
    start: &mut State,
    chars: Vec<char>,
) -> Result<(Vec<State>, usize, usize), String> {
    let q0 = generate_state(id_generator, true);
    let q0_id = q0.id;

    start.add_transition(TransitionKey::CharClass(chars), q0_id);

    Ok((vec![q0], q0_id, q0_id))
}

fn build_states(states: Vec<State>) -> HashMap<usize, State> {
    let mut map = HashMap::new();
    for state in states {
        map.insert(state.id, state);
    }
    map
}

#[allow(dead_code)]
pub fn match_nfa(nfa: &NFA, input_str: &str) -> Result<bool, String> {
    let mut input = InputWithIndex {
        index: 0,
        input: input_str.to_string(),
        visited: HashSet::new(),
    };
    let result = _match_nfa(nfa, nfa.start_id, &mut input)?;
    match result {
        MatchResult::Match => Ok(true),
        MatchResult::NoMatch => Ok(false),
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum MatchResult {
    Match,
    NoMatch,
}

#[derive(Debug)]
struct InputWithIndex {
    index: usize,
    input: String,
    visited: HashSet<(usize, usize)>,
}

impl InputWithIndex {
    fn new(input: String) -> Self {
        Self {
            index: 0,
            input,
            visited: HashSet::new(),
        }
    }

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
    if input.visited.contains(&(current_state_id, input.index)) {
        return Ok(MatchResult::NoMatch);
    }
    input.visited.insert((current_state_id, input.index));

    if input.is_end() {
        let closure = epsilon_closure(nfa, current_state_id)?;

        for state_id in closure {
            // end of line
            let end_of_line_states = nfa
                .states
                .get(&state_id)
                .map(|state| {
                    let mut ids = HashSet::new();
                    if let Some(transitions) = state.transitions.get(&TransitionKey::End) {
                        ids.extend(transitions.iter().cloned());
                    }
                    ids
                })
                .unwrap_or_default();

            for end_of_line_state_id in end_of_line_states {
                let state = nfa.states.get(&end_of_line_state_id).unwrap();
                if state.is_accept {
                    return Ok(MatchResult::Match);
                }
                // 最初のaccept状態でない状態が見つかったら、他もチェックする必要がある
            }

            if nfa.states.get(&state_id).unwrap().is_accept {
                return Ok(MatchResult::Match);
            }
        }
        return Ok(MatchResult::NoMatch);
    }

    if let Some(c) = input.peek() {
        let _next_states = nfa.states.get(&current_state_id).map(|state| {
            let mut next_state_ids = HashSet::new();
            if let Some(transitions) = state.transitions.get(&TransitionKey::Literal(c)) {
                next_state_ids.extend(transitions.iter().cloned());
            }
            if let Some(transitions) = state.transitions.get(&TransitionKey::AnyChar) {
                next_state_ids.extend(transitions.iter().cloned());
            }
            let mut adapted_char_class_transitions = HashSet::new();
            for transition in state.transitions.iter() {
                if let TransitionKey::CharClass(ref chars) = transition.0 {
                    if chars.contains(&c) {
                        adapted_char_class_transitions.extend(transition.1.iter().cloned());
                    }
                }
            }
            next_state_ids.extend(adapted_char_class_transitions);
            next_state_ids
        });

        let closure = epsilon_closure(nfa, current_state_id)?;

        let start_of_line_states = nfa
            .states
            .get(&current_state_id)
            .map(|state| {
                let mut ids = HashSet::new();
                if let Some(transitions) = state.transitions.get(&TransitionKey::Start) {
                    if input.index == 0 {
                        ids.extend(transitions.iter().cloned());
                    }
                }
                ids
            })
            .unwrap_or_default();
        let closure = closure.union(&start_of_line_states).cloned().collect();

        let next_states: HashSet<usize> = _next_states
            .unwrap_or_default()
            .union(&closure)
            .cloned()
            .collect();
        let next_states: HashSet<usize> =
            next_states.union(&start_of_line_states).cloned().collect();

        let next_states = next_states
            .into_iter()
            .map(|state_id| (state_id, closure.contains(&state_id)))
            .collect::<Vec<(usize, bool)>>();

        if next_states.is_empty() {
            input.next();
            return _match_nfa(nfa, nfa.start_id, input);
        }

        for (next_state_id, is_epsilon) in next_states {
            let next_state = nfa.states.get(&next_state_id).unwrap();

            if next_state.is_accept {
                return Ok(MatchResult::Match);
            } else {
                let current_index = input.index;
                if !is_epsilon {
                    input.next();
                }
                let result = _match_nfa(nfa, next_state_id, input)?;
                match result {
                    MatchResult::Match => return Ok(MatchResult::Match),
                    MatchResult::NoMatch => {
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
    let epsilon_states = current_state
        .transitions
        .get(&TransitionKey::Epsilon)
        .unwrap_or(&binding);
    for next_state_id in epsilon_states {
        if visited.contains(next_state_id) {
            continue;
        }
        visited.insert(*next_state_id);
        _epsilon_closure(nfa, *next_state_id, visited)?;
    }
    Ok(())
}

impl NFA {
    #[allow(dead_code)]
    pub fn to_dot(&self) -> String {
        format!(
            "digraph finite_state_machine {{\n{}\n}}",
            self.to_dot_body()
        )
    }

    fn to_dot_body(&self) -> String {
        let mut body = String::new();

        let mut accept_states = vec![];
        for state in self.states.values() {
            if state.is_accept {
                accept_states.push(state.id);
            }
        }

        body.push_str("\trankdir=LR\n");
        body.push_str(&format!(
            "\tnode [shape=doublecircle]; {};\n",
            accept_states
                .iter()
                .map(|id| format!("{}", id))
                .collect::<Vec<String>>()
                .join(" ")
        ));
        body.push_str("\tnode [shape=circle];\n");
        for state in self.states.values() {
            for (c, next_states) in state.transitions.iter() {
                for next_state_id in next_states {
                    body.push_str(&format!(
                        "\t{} -> {} [label=\"{}\"]\n",
                        state.id,
                        next_state_id,
                        match c {
                            TransitionKey::Literal(c) => format!("{}", c),
                            TransitionKey::Epsilon => "ε".to_string(),
                            TransitionKey::CharClass(chars) =>
                                format!("[{}]", chars.iter().collect::<String>()),
                            TransitionKey::AnyChar => "AnyChar".to_string(),
                            TransitionKey::Start => "^".to_string(),
                            TransitionKey::End => "$".to_string(),
                        }
                    ));
                }
            }
        }
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epsilon_closure() {
        let start_id = 0;

        // 0 -> 1 -> 2 -> 3
        let q0 = State::new(
            start_id,
            HashMap::from([(TransitionKey::Epsilon, HashSet::from([1]))]),
            false,
        );
        let q1 = State::new(
            1,
            HashMap::from([(TransitionKey::Literal('a'), HashSet::from([2]))]),
            false,
        );
        let q2 = State::new(
            2,
            HashMap::from([(TransitionKey::Literal('b'), HashSet::from([3]))]),
            false,
        );
        let q3 = State::new(3, HashMap::new(), true);
        let states = build_states(vec![q0, q1, q2, q3]);

        let nfa = NFA { start_id, states };
        let result = epsilon_closure(&nfa, 0);
        assert_eq!(result, Ok(HashSet::from([1])));

        //      <---
        //      |  |
        // 0 -> 1 ->-> 2 -> 3
        let q0 = State::new(
            start_id,
            HashMap::from([(TransitionKey::Epsilon, HashSet::from([1]))]),
            false,
        );
        let q1 = State::new(
            1,
            HashMap::from([
                (TransitionKey::Literal('a'), HashSet::from([2])),
                (TransitionKey::Epsilon, HashSet::from([1])),
            ]),
            false,
        );
        let q2 = State::new(
            2,
            HashMap::from([(TransitionKey::Literal('b'), HashSet::from([3]))]),
            false,
        );
        let q3 = State::new(3, HashMap::new(), true);
        let states = build_states(vec![q0, q1, q2, q3]);
        let nfa = NFA { start_id, states };

        let result = epsilon_closure(&nfa, 0);
        assert_eq!(result, Ok(HashSet::from([1])));

        let result = epsilon_closure(&nfa, 1);
        assert_eq!(result, Ok(HashSet::from([1])));

        let result = epsilon_closure(&nfa, 2);
        assert_eq!(result, Ok(HashSet::from([])));

        // 0 -> 1 -> 2 -> 3
        let q0 = State::new(
            start_id,
            HashMap::from([(TransitionKey::Epsilon, HashSet::from([1]))]),
            false,
        );
        let q1 = State::new(
            1,
            HashMap::from([
                (TransitionKey::Literal('a'), HashSet::from([2])),
                (TransitionKey::Epsilon, HashSet::from([0, 1])),
            ]),
            false,
        );
        let q2 = State::new(
            2,
            HashMap::from([(TransitionKey::Literal('b'), HashSet::from([3]))]),
            false,
        );
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
        assert_eq!(match_nfa(&nfa, "aab"), Ok(false));
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

        // a*
        let nfa = build_nfa(Node::ZeroOrMore(Box::new(Node::Literal('a')))).unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "aa"), Ok(true));
        assert_eq!(match_nfa(&nfa, ""), Ok(true));
        assert_eq!(match_nfa(&nfa, "b"), Ok(true));

        // a*b
        let nfa = build_nfa(Node::Concat(vec![
            Node::ZeroOrMore(Box::new(Node::Literal('a'))),
            Node::Literal('b'),
        ]))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "ab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "aab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "b"), Ok(true));
        assert_eq!(match_nfa(&nfa, "bb"), Ok(true));
        assert_eq!(match_nfa(&nfa, "a"), Ok(false));

        // a+
        let nfa = build_nfa(Node::OneOrMore(Box::new(Node::Literal('a')))).unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "aa"), Ok(true));
        assert_eq!(match_nfa(&nfa, "aaa"), Ok(true));
        assert_eq!(match_nfa(&nfa, ""), Ok(false));
        assert_eq!(match_nfa(&nfa, "b"), Ok(false));

        // a+b
        let nfa = build_nfa(Node::Concat(vec![
            Node::OneOrMore(Box::new(Node::Literal('a'))),
            Node::Literal('b'),
        ]))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "ab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "aab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "aaab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "b"), Ok(false));

        // a?
        let nfa = build_nfa(Node::ZeroOrOne(Box::new(Node::Literal('a')))).unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "aa"), Ok(true));
        assert_eq!(match_nfa(&nfa, ""), Ok(true));
        assert_eq!(match_nfa(&nfa, "b"), Ok(true));

        // a?b
        let nfa = build_nfa(Node::Concat(vec![
            Node::ZeroOrOne(Box::new(Node::Literal('a'))),
            Node::Literal('b'),
        ]))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "ab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "b"), Ok(true));
        assert_eq!(match_nfa(&nfa, "aa"), Ok(false));
        assert_eq!(match_nfa(&nfa, "aab"), Ok(true));

        // (a)
        let nfa = build_nfa(Node::Group(Box::new(Node::Literal('a')))).unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "aa"), Ok(true));
        assert_eq!(match_nfa(&nfa, "b"), Ok(false));

        // (a|b)
        let nfa = build_nfa(Node::Group(Box::new(Node::Or(
            Box::new(Node::Literal('a')),
            Box::new(Node::Literal('b')),
        ))))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "b"), Ok(true));

        // (ab)*
        let nfa = build_nfa(Node::ZeroOrMore(Box::new(Node::Concat(vec![
            Node::Literal('a'),
            Node::Literal('b'),
        ]))))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "ab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "abab"), Ok(true));

        // (ab)*c
        let nfa = build_nfa(Node::Concat(vec![
            Node::ZeroOrMore(Box::new(Node::Concat(vec![
                Node::Literal('a'),
                Node::Literal('b'),
            ]))),
            Node::Literal('c'),
        ]))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "ab"), Ok(false));
        assert_eq!(match_nfa(&nfa, "abab"), Ok(false));
        assert_eq!(match_nfa(&nfa, "abc"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ababc"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ac"), Ok(true));
        assert_eq!(match_nfa(&nfa, "bc"), Ok(true));

        // .
        let nfa = build_nfa(Node::AnyChar).unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "abc"), Ok(true));
        assert_eq!(match_nfa(&nfa, ""), Ok(false));

        // .bc
        let nfa = build_nfa(Node::Concat(vec![
            Node::AnyChar,
            Node::Literal('b'),
            Node::Literal('c'),
        ]))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "abc"), Ok(true));
        assert_eq!(match_nfa(&nfa, "bbc"), Ok(true));
        assert_eq!(match_nfa(&nfa, "bcc"), Ok(false));
        assert_eq!(match_nfa(&nfa, ""), Ok(false));
        assert_eq!(match_nfa(&nfa, "a"), Ok(false));
        assert_eq!(match_nfa(&nfa, "b"), Ok(false));
        assert_eq!(match_nfa(&nfa, "c"), Ok(false));

        // [a-c]
        let nfa = build_nfa(Node::CharClass(vec!['a', 'b', 'c'])).unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "b"), Ok(true));
        assert_eq!(match_nfa(&nfa, "c"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ab"), Ok(true));
        assert_eq!(match_nfa(&nfa, "bc"), Ok(true));
        assert_eq!(match_nfa(&nfa, "abc"), Ok(true));
        assert_eq!(match_nfa(&nfa, ""), Ok(false));
        assert_eq!(match_nfa(&nfa, "d"), Ok(false));
        assert_eq!(match_nfa(&nfa, "da"), Ok(true));

        // [a-c]d
        let nfa = build_nfa(Node::Concat(vec![
            Node::CharClass(vec!['a', 'b', 'c']),
            Node::Literal('d'),
        ]))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "ad"), Ok(true));
        assert_eq!(match_nfa(&nfa, "bd"), Ok(true));
        assert_eq!(match_nfa(&nfa, "cd"), Ok(true));
        assert_eq!(match_nfa(&nfa, "dd"), Ok(false));

        // ^a
        let nfa = build_nfa(Node::Concat(vec![Node::Start, Node::Literal('a')])).unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ba"), Ok(false));
        assert_eq!(match_nfa(&nfa, ""), Ok(false));

        // a$
        let nfa = build_nfa(Node::Concat(vec![Node::Literal('a'), Node::End])).unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ba"), Ok(true));
        assert_eq!(match_nfa(&nfa, ""), Ok(false));

        // ^a$
        let nfa = build_nfa(Node::Concat(vec![
            Node::Start,
            Node::Literal('a'),
            Node::End,
        ]))
        .unwrap();
        assert_eq!(match_nfa(&nfa, "a"), Ok(true));
        assert_eq!(match_nfa(&nfa, "ba"), Ok(false));
        assert_eq!(match_nfa(&nfa, ""), Ok(false));
    }
}
