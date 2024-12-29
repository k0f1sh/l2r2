mod lexer;
mod nfa;
mod parser;

use lexer::lex;
use nfa::{build_nfa, match_nfa};
use parser::{parse, Node};

fn main() {
    println!("Hello, world!");

    let l = lex("a|b").unwrap();
    let p = parse(l).unwrap();

    let nfa = build_nfa(Node::Literal('a')).unwrap();
    println!("{:#?}", nfa);

    let result = match_nfa(&nfa, "a|b");
    println!("input: ba, result: {}", result);
}
