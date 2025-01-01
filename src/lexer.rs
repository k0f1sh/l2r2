#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    Literal(char), // single character
    Star,          // *
    Plus,          // +
    Question,      // ?
    Dot,           // .
    Hyphen,        // -
    Pipe,          // |
    LeftParen,     // (
    RightParen,    // )
    LeftBracket,   // [
    RightBracket,  // ]
                   // TODO: Add more tokens
}

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut it = input.chars().peekable();
    while let Some(&c) = it.peek() {
        match c {
            '*' => tokens.push(Token::Star),
            '+' => tokens.push(Token::Plus),
            '?' => tokens.push(Token::Question),
            '.' => tokens.push(Token::Dot),
            '-' => tokens.push(Token::Hyphen),
            '|' => tokens.push(Token::Pipe),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            '[' => tokens.push(Token::LeftBracket),
            ']' => tokens.push(Token::RightBracket),
            '\\' => {
                it.next().unwrap();
                tokens.push(Token::Literal(it.peek().unwrap().clone()));
            }
            _ => {
                if c.is_ascii_alphabetic() || c.is_ascii_digit() {
                    tokens.push(Token::Literal(c));
                } else if !c.is_whitespace() {
                    return Err(format!("Invalid character: {}", c));
                }
            }
        }
        it.next();
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        assert_eq!(lex("a").unwrap(), vec![Token::Literal('a')]);
        assert_eq!(lex("Z").unwrap(), vec![Token::Literal('Z')]);
        assert_eq!(
            lex("aa").unwrap(),
            vec![Token::Literal('a'), Token::Literal('a')]
        );
        assert_eq!(lex("a*").unwrap(), vec![Token::Literal('a'), Token::Star]);
        assert_eq!(lex("a+").unwrap(), vec![Token::Literal('a'), Token::Plus]);
        assert_eq!(
            lex("a?").unwrap(),
            vec![Token::Literal('a'), Token::Question]
        );
        assert_eq!(lex("a.").unwrap(), vec![Token::Literal('a'), Token::Dot]);
        assert_eq!(
            lex("a(b)").unwrap(),
            vec![
                Token::Literal('a'),
                Token::LeftParen,
                Token::Literal('b'),
                Token::RightParen
            ]
        );
        assert_eq!(lex(".*").unwrap(), vec![Token::Dot, Token::Star]);
        assert_eq!(
            lex("[a-z]").unwrap(),
            vec![
                Token::LeftBracket,
                Token::Literal('a'),
                Token::Hyphen,
                Token::Literal('z'),
                Token::RightBracket
            ]
        );
        assert_eq!(
            lex(" a-z ").unwrap(),
            vec![Token::Literal('a'), Token::Hyphen, Token::Literal('z')]
        );
        assert_eq!(
            lex("a|b").unwrap(),
            vec![Token::Literal('a'), Token::Pipe, Token::Literal('b')]
        );
        assert_eq!(lex("\\a").unwrap(), vec![Token::Literal('a')]);
        assert_eq!(lex("\\-").unwrap(), vec![Token::Literal('-')]);
    }
}
