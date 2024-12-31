mod lexer;
mod nfa;
mod parser;

use lexer::lex;
use nfa::{build_nfa, match_nfa};
use parser::parse;

fn main() {
    println!("Hello, world!");

    let input = "abcd";

    let l = lex("abcd").unwrap();
    let p = parse(l).unwrap();
    println!("--- parse ---");
    println!("{:#?}", p);

    let nfa = build_nfa(p).unwrap();
    println!("--- nfa ---");
    println!("{:#?}", nfa);
    let result = match_nfa(&nfa, input);
    println!("--- result ---");
    println!("{:#?}", result);
}
