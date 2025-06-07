#![allow(dead_code)]

use crate::nfa::{TransitionKey, NFA};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DFAState {
    pub id: usize,
    pub nfa_states: BTreeSet<usize>, // このDFA状態が表すNFA状態の集合
    pub transitions: HashMap<TransitionKey, usize>, // 文字 -> 次のDFA状態ID
    pub is_accept: bool,
}

impl DFAState {
    pub fn new(id: usize, nfa_states: BTreeSet<usize>, is_accept: bool) -> Self {
        Self {
            id,
            nfa_states,
            transitions: HashMap::new(),
            is_accept,
        }
    }
}

#[derive(Debug)]
pub struct DFA {
    pub start_id: usize,
    pub states: HashMap<usize, DFAState>,
}

impl DFA {
    pub fn new(start_id: usize, states: HashMap<usize, DFAState>) -> Self {
        Self { start_id, states }
    }
}

/// NFAからDFAに変換する関数
/// サブセット構築法（Subset Construction）を使用
pub fn nfa_to_dfa(nfa: &NFA) -> Result<DFA, String> {
    let mut dfa_states: HashMap<usize, DFAState> = HashMap::new();
    let mut state_map: HashMap<BTreeSet<usize>, usize> = HashMap::new(); // NFA状態集合 -> DFA状態ID
    let mut work_queue: VecDeque<BTreeSet<usize>> = VecDeque::new();
    let mut next_dfa_id = 0;

    // 1. 開始状態のε-closureを計算
    let start_closure_set = epsilon_closure_set(nfa, &HashSet::from([nfa.start_id]))?;
    let start_closure: BTreeSet<usize> = start_closure_set.into_iter().collect();

    // 2. 開始状態がacceptかどうかチェック
    let start_is_accept = start_closure
        .iter()
        .any(|&state_id| nfa.states.get(&state_id).is_some_and(|s| s.is_accept));

    // 3. 開始DFA状態を作成
    let start_dfa_state = DFAState::new(next_dfa_id, start_closure.clone(), start_is_accept);
    dfa_states.insert(next_dfa_id, start_dfa_state);
    state_map.insert(start_closure.clone(), next_dfa_id);
    work_queue.push_back(start_closure);
    let start_id = next_dfa_id;
    next_dfa_id += 1;

    // 4. 全ての状態を処理するまでループ
    while let Some(current_nfa_states) = work_queue.pop_front() {
        let current_dfa_id = *state_map.get(&current_nfa_states).unwrap();

        // 5. 各文字に対して遷移先を計算
        let mut transitions_by_char: HashMap<TransitionKey, HashSet<usize>> = HashMap::new();

        for &nfa_state_id in &current_nfa_states {
            if let Some(nfa_state) = nfa.states.get(&nfa_state_id) {
                for (transition_key, target_states) in &nfa_state.transitions {
                    // ε遷移は既にclosureで処理済みなのでスキップ
                    if *transition_key == TransitionKey::Epsilon {
                        continue;
                    }

                    transitions_by_char
                        .entry(transition_key.clone())
                        .or_default()
                        .extend(target_states);
                }
            }
        }

        // 6. 各遷移に対してε-closureを計算し、新しいDFA状態を作成
        for (transition_key, target_nfa_states) in transitions_by_char {
            let target_closure_set = epsilon_closure_set(nfa, &target_nfa_states)?;
            let target_closure: BTreeSet<usize> = target_closure_set.into_iter().collect();

            // 7. この状態集合が既に存在するかチェック
            let target_dfa_id = if let Some(&existing_id) = state_map.get(&target_closure) {
                existing_id
            } else {
                // 8. 新しいDFA状態を作成
                let target_is_accept = target_closure
                    .iter()
                    .any(|&state_id| nfa.states.get(&state_id).is_some_and(|s| s.is_accept));

                let new_dfa_state =
                    DFAState::new(next_dfa_id, target_closure.clone(), target_is_accept);
                dfa_states.insert(next_dfa_id, new_dfa_state);
                state_map.insert(target_closure.clone(), next_dfa_id);
                work_queue.push_back(target_closure);

                let id = next_dfa_id;
                next_dfa_id += 1;
                id
            };

            // 9. 遷移を追加
            if let Some(current_dfa_state) = dfa_states.get_mut(&current_dfa_id) {
                current_dfa_state
                    .transitions
                    .insert(transition_key, target_dfa_id);
            }
        }
    }

    Ok(DFA::new(start_id, dfa_states))
}

/// 状態集合のε-closureを計算する
fn epsilon_closure_set(nfa: &NFA, states: &HashSet<usize>) -> Result<HashSet<usize>, String> {
    let mut closure = states.clone();
    let mut work_queue: VecDeque<usize> = states.iter().cloned().collect();

    while let Some(current_state_id) = work_queue.pop_front() {
        if let Some(current_state) = nfa.states.get(&current_state_id) {
            if let Some(epsilon_targets) = current_state.transitions.get(&TransitionKey::Epsilon) {
                for &target_id in epsilon_targets {
                    if !closure.contains(&target_id) {
                        closure.insert(target_id);
                        work_queue.push_back(target_id);
                    }
                }
            }
        }
    }

    Ok(closure)
}

/// DFAでマッチングを実行する（NFAより高速）
/// 部分マッチに対応（文字列の任意の位置からマッチを試行）
pub fn match_dfa(dfa: &DFA, input: &str) -> Result<bool, String> {
    let chars: Vec<char> = input.chars().collect();

    // 文字列の各位置からマッチを試行
    for start_pos in 0..=chars.len() {
        if try_match_from_position(dfa, &chars, start_pos)? {
            return Ok(true);
        }
    }

    Ok(false)
}

/// 指定された位置からDFAマッチングを試行
fn try_match_from_position(dfa: &DFA, chars: &[char], start_pos: usize) -> Result<bool, String> {
    let mut current_state_id = dfa.start_id;

    for i in start_pos..chars.len() {
        let c = chars[i];

        if let Some(current_state) = dfa.states.get(&current_state_id) {
            // 現在の状態がaccept状態なら、ここでマッチ成功
            if current_state.is_accept {
                return Ok(true);
            }

            // 文字に対応する遷移を探す
            let next_state_id = current_state
                .transitions
                .get(&TransitionKey::Literal(c))
                .or_else(|| current_state.transitions.get(&TransitionKey::AnyChar))
                .or_else(|| {
                    // 文字クラスをチェック
                    for (key, target_id) in &current_state.transitions {
                        if let TransitionKey::CharClass(chars) = key {
                            if chars.contains(&c) {
                                return Some(target_id);
                            }
                        }
                    }
                    None
                });

            if let Some(&next_id) = next_state_id {
                current_state_id = next_id;
            } else {
                // 遷移が見つからない場合は、この位置からのマッチは失敗
                return Ok(false);
            }
        } else {
            return Err(format!("Invalid state: {}", current_state_id));
        }
    }

    // 文字列の最後まで到達した場合、最終状態がaccept状態かチェック
    if let Some(final_state) = dfa.states.get(&current_state_id) {
        Ok(final_state.is_accept)
    } else {
        Err(format!("Invalid final state: {}", current_state_id))
    }
}

impl DFA {
    /// DFAをDOT形式で出力（可視化用）
    pub fn to_dot(&self) -> String {
        format!("digraph DFA {{\n{}\n}}", self.to_dot_body())
    }

    fn to_dot_body(&self) -> String {
        let mut body = String::new();

        // accept状態を特別な形で表示
        let accept_states: Vec<usize> = self
            .states
            .values()
            .filter(|state| state.is_accept)
            .map(|state| state.id)
            .collect();

        body.push_str("\trankdir=LR\n");
        if !accept_states.is_empty() {
            body.push_str(&format!(
                "\tnode [shape=doublecircle]; {};\n",
                accept_states
                    .iter()
                    .map(|id| format!("{}", id))
                    .collect::<Vec<String>>()
                    .join(" ")
            ));
        }
        body.push_str("\tnode [shape=circle];\n");

        // 遷移を出力
        for state in self.states.values() {
            for (transition_key, &target_id) in &state.transitions {
                let label = match transition_key {
                    TransitionKey::Literal(c) => format!("{}", c),
                    TransitionKey::AnyChar => ".".to_string(),
                    TransitionKey::CharClass(chars) => {
                        format!("[{}]", chars.iter().collect::<String>())
                    }
                    TransitionKey::Start => "^".to_string(),
                    TransitionKey::End => "$".to_string(),
                    TransitionKey::Epsilon => "ε".to_string(), // DFAにはないはずだが念のため
                };

                body.push_str(&format!(
                    "\t{} -> {} [label=\"{}\"];\n",
                    state.id, target_id, label
                ));
            }
        }

        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nfa::build_nfa;
    use crate::parser::Node;

    #[test]
    fn test_simple_dfa_conversion() {
        // "a" のNFAを作成してDFAに変換
        let nfa = build_nfa(Node::Literal('a')).unwrap();
        let dfa = nfa_to_dfa(&nfa).unwrap();

        // DFAでマッチングテスト（部分マッチ）
        assert_eq!(match_dfa(&dfa, "a"), Ok(true));
        assert_eq!(match_dfa(&dfa, "b"), Ok(false));
        assert_eq!(match_dfa(&dfa, "aa"), Ok(true)); // 部分マッチなので"a"が含まれてればOK
        assert_eq!(match_dfa(&dfa, "ba"), Ok(true)); // "a"が含まれてる
        assert_eq!(match_dfa(&dfa, "abc"), Ok(true)); // "a"が含まれてる
    }

    #[test]
    fn test_or_dfa_conversion() {
        // "a|b" のNFAを作成してDFAに変換
        let nfa = build_nfa(Node::Or(
            Box::new(Node::Literal('a')),
            Box::new(Node::Literal('b')),
        ))
        .unwrap();
        let dfa = nfa_to_dfa(&nfa).unwrap();

        // DFAでマッチングテスト
        assert_eq!(match_dfa(&dfa, "a"), Ok(true));
        assert_eq!(match_dfa(&dfa, "b"), Ok(true));
        assert_eq!(match_dfa(&dfa, "c"), Ok(false));
    }

    #[test]
    fn test_concat_dfa_conversion() {
        // "ab" のNFAを作成してDFAに変換
        let nfa = build_nfa(Node::Concat(vec![Node::Literal('a'), Node::Literal('b')])).unwrap();
        let dfa = nfa_to_dfa(&nfa).unwrap();

        // DFAでマッチングテスト（部分マッチ）
        assert_eq!(match_dfa(&dfa, "ab"), Ok(true));
        assert_eq!(match_dfa(&dfa, "a"), Ok(false)); // "ab"パターンが含まれてない
        assert_eq!(match_dfa(&dfa, "b"), Ok(false)); // "ab"パターンが含まれてない
        assert_eq!(match_dfa(&dfa, "abc"), Ok(true)); // "ab"が含まれてる
        assert_eq!(match_dfa(&dfa, "cab"), Ok(true)); // "ab"が含まれてる
        assert_eq!(match_dfa(&dfa, "cabc"), Ok(true)); // "ab"が含まれてる
    }
}
