mod lexer;
mod nfa;
mod parser;

use std::io::BufRead;
use std::{env, io};

use lexer::lex;
use nfa::{build_nfa, match_nfa};
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

    let stdin = io::stdin();
    let handle = stdin.lock();
    let lines = handle.lines();

    for line in lines {
        match line {
            Ok(input) => {
                let result = match_nfa(&nfa, &input);
                let matched = result.unwrap();
                if matched {
                    println!("{}", input);
                    continue;
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}
