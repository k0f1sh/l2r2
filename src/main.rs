mod lexer;
mod nfa;
mod parser;

use lexer::lex;
use nfa::{build_nfa, match_nfa};
use parser::{parse, Node};

fn main() {
    println!("Hello, world!");

    let l = lex("a|([a-c])").unwrap();
    let p = parse(l).unwrap();
    println!("{:#?}", p)
}
