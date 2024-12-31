mod lexer;
mod nfa;
mod parser;

use std::env;

use lexer::lex;
use nfa::build_nfa;
use parser::parse;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <regex>", args[0]);
        return;
    }

    let regex = args[1].clone();
    let l = lex(&regex).unwrap();
    let p = parse(l).unwrap();
    let nfa = build_nfa(p).unwrap();

    println!("{}", nfa.to_dot());
}
