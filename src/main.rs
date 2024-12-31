mod lexer;
mod nfa;
mod parser;

use lexer::lex;
use nfa::{build_nfa, match_nfa};
use parser::parse;

fn main() {
    println!("Hello, world!");

    let l = lex("a|([a-c])").unwrap();
    let p = parse(l).unwrap();

    let nfa = build_nfa(p);
    println!("--- nfa ---");
    println!("{:#?}", nfa);
    let result = match_nfa(&nfa, "bab");
    println!("--- result ---");
    println!("{:#?}", result);
}
