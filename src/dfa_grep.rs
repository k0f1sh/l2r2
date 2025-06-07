mod dfa;
mod lexer;
mod nfa;
mod parser;

use std::io::BufRead;
use std::{env, io};

use dfa::{match_dfa, nfa_to_dfa};
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

    // 1. Lexer: 正規表現をトークンに分割
    let tokens = match lex(&regex) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            return;
        }
    };

    // 2. Parser: トークンをASTに変換
    let ast = match parse(tokens) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parser error: {}", e);
            return;
        }
    };

    // 3. NFA Builder: ASTからNFAを構築
    let nfa = match build_nfa(ast) {
        Ok(nfa) => nfa,
        Err(e) => {
            eprintln!("NFA build error: {}", e);
            return;
        }
    };

    // 4. DFA Converter: NFAからDFAに変換
    let dfa = match nfa_to_dfa(&nfa) {
        Ok(dfa) => dfa,
        Err(e) => {
            eprintln!("DFA conversion error: {}", e);
            return;
        }
    };

    println!("DFA conversion successful! States: {}", dfa.states.len());

    // 5. 標準入力からテキストを読み込んでマッチング
    let stdin = io::stdin();
    let handle = stdin.lock();
    let lines = handle.lines();

    for line in lines {
        match line {
            Ok(input) => {
                let result = match_dfa(&dfa, &input);
                match result {
                    Ok(matched) => {
                        if matched {
                            println!("{}", input);
                        }
                    }
                    Err(e) => {
                        eprintln!("DFA match error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Input error: {}", e);
            }
        }
    }
}
