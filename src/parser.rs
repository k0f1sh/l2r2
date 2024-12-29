use std::iter::Peekable;

use crate::lexer::Token;

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    Literal(char),
    AnyChar,
    CharClass(Vec<char>),
    Union(Box<Node>, Box<Node>),
    ZeroOrMore(Box<Node>),
    OneOrMore(Box<Node>),
    ZeroOrOne(Box<Node>),
    Group(Box<Node>),
    Concat(Vec<Node>),
}

pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let mut tokens = tokens.into_iter().peekable();
    parse_expr(&mut tokens)
}

fn parse_expr(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let mut left = parse_term(tokens)?;

    while let Some(token) = tokens.peek() {
        match token {
            Token::Pipe => {
                tokens.next();
                let right = parse_term(tokens)?;
                left = Node::Union(Box::new(left), Box::new(right));
            }
            _ => break,
        }
    }

    Ok(left)
}

fn parse_term(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let mut nodes = Vec::new();
    while let Some(token) = tokens.peek() {
        match token {
            Token::Literal(_) | Token::Dot | Token::LeftParen | Token::LeftBracket => {
                nodes.push(parse_factor(tokens)?);
            }
            Token::Pipe | Token::RightParen => {
                break;
            }
            _ => {
                return Err(format!("Unexpected token: {:?}", token));
            }
        }
    }

    if nodes.len() == 1 {
        Ok(nodes.pop().unwrap())
    } else {
        Ok(Node::Concat(nodes))
    }
}

fn parse_factor(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let token = tokens.next().unwrap();
    let node = match token {
        Token::Literal(c) => Ok(Node::Literal(c)),
        Token::Dot => Ok(Node::AnyChar),
        Token::LeftParen => {
            let expr = parse_expr(tokens)?;
            if let Some(Token::RightParen) = tokens.next() {
                Ok(Node::Group(Box::new(expr)))
            } else {
                Err(format!("Unclosed group"))
            }
        }
        Token::LeftBracket => {
            let expr = parse_char_class(tokens)?;
            Ok(expr)
        }
        _ => Err(format!("Unexpected token: {:?}", token)),
    }?;

    if let Some(_) = tokens.peek() {
        parse_repetition(tokens, node)
    } else {
        Ok(node)
    }
}

fn parse_repetition(
    tokens: &mut Peekable<impl Iterator<Item = Token>>,
    node: Node,
) -> Result<Node, String> {
    let token = tokens.peek().unwrap();
    match token {
        Token::Star => {
            tokens.next();
            Ok(Node::ZeroOrMore(Box::new(node)))
        }
        Token::Plus => {
            tokens.next();
            Ok(Node::OneOrMore(Box::new(node)))
        }
        Token::Question => {
            tokens.next();
            Ok(Node::ZeroOrOne(Box::new(node)))
        }
        _ => Ok(node),
    }
}

fn parse_char_class(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let mut chars = Vec::new();
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
            Token::RightBracket => {
                break;
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
            parse(lex("a*").unwrap()),
            Ok(Node::ZeroOrMore(Box::new(Node::Literal('a'))))
        );
        assert_eq!(
            parse(lex("a+").unwrap()),
            Ok(Node::OneOrMore(Box::new(Node::Literal('a'))))
        );
        assert_eq!(
            parse(lex("a?").unwrap()),
            Ok(Node::ZeroOrOne(Box::new(Node::Literal('a'))))
        );
        assert_eq!(
            parse(lex("a*b").unwrap()),
            Ok(Node::Concat(vec![
                Node::ZeroOrMore(Box::new(Node::Literal('a'))),
                Node::Literal('b')
            ]))
        );
        assert_eq!(parse(lex(".").unwrap()), Ok(Node::AnyChar));
        assert_eq!(
            parse(lex(".*").unwrap()),
            Ok(Node::ZeroOrMore(Box::new(Node::AnyChar)))
        );
        assert_eq!(
            parse(lex("a|b").unwrap()),
            Ok(Node::Union(
                Box::new(Node::Literal('a')),
                Box::new(Node::Literal('b'))
            ))
        );
        assert_eq!(
            parse(lex("a|b|c").unwrap()),
            Ok(Node::Union(
                Box::new(Node::Union(
                    Box::new(Node::Literal('a')),
                    Box::new(Node::Literal('b'))
                )),
                Box::new(Node::Literal('c'))
            ))
        );
        assert_eq!(
            parse(lex("a*|b").unwrap()),
            Ok(Node::Union(
                Box::new(Node::ZeroOrMore(Box::new(Node::Literal('a')))),
                Box::new(Node::Literal('b'))
            ))
        );
        assert_eq!(
            parse(lex("a|([a-c])").unwrap()),
            Ok(Node::Union(
                Box::new(Node::Literal('a')),
                Box::new(Node::Group(Box::new(Node::CharClass(vec!['a', 'b', 'c']))))
            ))
        );
        assert_eq!(
            parse(lex("abc|d").unwrap()),
            Ok(Node::Union(
                Box::new(Node::Concat(vec![
                    Node::Literal('a'),
                    Node::Literal('b'),
                    Node::Literal('c')
                ])),
                Box::new(Node::Literal('d'))
            ))
        );
        assert_eq!(
            parse(lex("(ab)*").unwrap()),
            Ok(Node::ZeroOrMore(Box::new(Node::Group(Box::new(
                Node::Concat(vec![Node::Literal('a'), Node::Literal('b')])
            )))))
        );
        assert_eq!(
            parse(lex("[abc]").unwrap()),
            Ok(Node::CharClass(vec!['a', 'b', 'c']))
        );
        assert_eq!(
            parse(lex("[abc]*").unwrap()),
            Ok(Node::ZeroOrMore(Box::new(Node::CharClass(vec![
                'a', 'b', 'c'
            ]))))
        );
        assert_eq!(
            parse(lex("[abc]+").unwrap()),
            Ok(Node::OneOrMore(Box::new(Node::CharClass(vec![
                'a', 'b', 'c'
            ]))))
        );
        assert_eq!(
            parse(lex("([a-c])").unwrap()),
            Ok(Node::Group(Box::new(Node::CharClass(vec!['a', 'b', 'c']))))
        );
        assert_eq!(
            parse(lex("[(a-c)]").unwrap()),
            Err("Unexpected token: LeftParen".to_string())
        );
    }
}
