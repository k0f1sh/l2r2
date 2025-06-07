mod dfa;
mod lexer;
mod nfa;
mod parser;

use std::env;

use dfa::nfa_to_dfa;
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

    // 5. DFAをDOT形式で出力
    println!("{}", dfa.to_dot());
}
