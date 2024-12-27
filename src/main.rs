mod lexer;
mod nfa;
mod parser;

use nfa::{build_nfa, match_nfa};
use parser::Node;

fn main() {
    println!("Hello, world!");

    let nfa = build_nfa(Node::Literal('a')).unwrap();
    println!("{:#?}", nfa);

    let result = match_nfa(&nfa, "a");
    println!("input: a, result: {}", result);

    let result = match_nfa(&nfa, "ab");
    println!("input: ab, result: {}", result);

    let result = match_nfa(&nfa, "ba");
    println!("input: ba, result: {}", result);
}
