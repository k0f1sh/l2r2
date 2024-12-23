use std::iter::Peekable;

use crate::lexer::Token;

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    Literal(char),
    AnyChar,
    CharClass(Vec<char>),
    ZeroOrMore(Box<Node>),
    OneOrMore(Box<Node>),
    ZeroOrOne(Box<Node>),
    Group(Box<Node>),
    Concat(Vec<Node>),
}

pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let mut tokens = tokens.into_iter().peekable();
    let mut nodes = Vec::new();
    while tokens.peek().is_some() {
        let token = tokens.next().unwrap();
        match token {
            Token::Literal(c) => nodes.push(Node::Literal(c)),
            Token::Star => {
                let prev = nodes
                    .pop()
                    .ok_or(format!("Unexpected token: {:?}", token))?;
                nodes.push(Node::ZeroOrMore(Box::new(prev)));
            }
            Token::Plus => {
                let prev = nodes
                    .pop()
                    .ok_or(format!("Unexpected token: {:?}", token))?;
                nodes.push(Node::OneOrMore(Box::new(prev)));
            }
            Token::Question => {
                let prev = nodes
                    .pop()
                    .ok_or(format!("Unexpected token: {:?}", token))?;
                nodes.push(Node::ZeroOrOne(Box::new(prev)));
            }
            Token::Dot => nodes.push(Node::AnyChar),
            Token::LeftParen => {
                // take all tokens until right paren
                let mut reached_right_paren = false;
                let mut sub_tokens = Vec::new();
                while tokens.peek().is_some() {
                    let token = tokens.next().unwrap();
                    if token == Token::LeftParen {
                        return Err(format!("Nested groups are not allowed"));
                    }
                    if token == Token::RightParen {
                        reached_right_paren = true;
                        break;
                    }
                    sub_tokens.push(token);
                }
                if !reached_right_paren {
                    return Err(format!("Unclosed group"));
                }
                nodes.push(Node::Group(Box::new(parse(sub_tokens)?)))
            }
            Token::LeftBracket => {
                // take all tokens until right paren
                let mut reached_right_bracket = false;
                let mut sub_tokens = Vec::new();
                while tokens.peek().is_some() {
                    let token = tokens.next().unwrap();
                    if token == Token::RightBracket {
                        reached_right_bracket = true;
                        break;
                    }
                    sub_tokens.push(token);
                }
                if !reached_right_bracket {
                    return Err(format!("Unclosed char class"));
                }
                nodes.push(parse_char_class(sub_tokens)?)
            }
            _ => return Err(format!("Unexpected token: {:?}", token)),
        }
    }

    if nodes.len() == 1 {
        return Ok(nodes.pop().unwrap());
    } else {
        return Ok(Node::Concat(nodes));
    };
}

fn parse_char_class(tokens: Vec<Token>) -> Result<Node, String> {
    let mut chars = Vec::new();
    let mut tokens = tokens.into_iter().peekable();
    while tokens.peek().is_some() {
        let token = tokens.next().unwrap();
        match token {
            Token::Literal(c) => chars.push(c),
            Token::Hyphen => {
                if chars.is_empty() {
                    return Err(format!("Hyphen at the beginning of char class"));
                }
                let next_token = tokens.next();
                if next_token.is_some() {
                    if let Some(Token::Literal(char)) = next_token {
                        let first = chars.pop().unwrap();
                        let last = char;
                        if first > last {
                            return Err(format!("invalid char class"));
                        }
                        for c in first..=last {
                            chars.push(c);
                        }
                    }

                    // e.g. [a-z-]
                    if let Some(Token::Hyphen) = tokens.peek() {
                        return Err(format!("invalid char class"));
                    }
                } else {
                    return Err(format!("invalid char class"));
                }
            }
            _ => return Err(format!("Unexpected token: {:?}", token)),
        }
    }
    Ok(Node::CharClass(chars))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[test]
    fn test_parse() {
        assert_eq!(parse(lex("a").unwrap()), Ok(Node::Literal('a')));
        assert_eq!(
            parse(lex("ab").unwrap()),
            Ok(Node::Concat(vec![Node::Literal('a'), Node::Literal('b')]))
        );
        assert_eq!(
            parse(lex("[abc]").unwrap()),
            Ok(Node::CharClass(vec!['a', 'b', 'c']))
        );
        assert_eq!(
            parse(lex("[a-c]").unwrap()),
            Ok(Node::CharClass(vec!['a', 'b', 'c']))
        );
    }
}
