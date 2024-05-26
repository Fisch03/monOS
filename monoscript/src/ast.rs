use alloc::{string::String, vec::Vec};

use super::{Token, TokenIterator};
use crate::interpret::Value;

#[derive(Debug)]
pub enum ParseError<'a> {
    UnexpectedToken(Token<'a>),
    UnexpectedEOF(&'static str),
}
impl<'a> Token<'a> {
    pub fn unexpected(self) -> ParseError<'a> {
        ParseError::UnexpectedToken(self)
    }
}

#[derive(Debug)]
pub struct ScriptAST<'a> {
    pub nodes: Vec<ASTNode<'a>>,
}
pub struct ASTIter<'a, 'b> {
    nodes: &'b Vec<ASTNode<'a>>,
    current_index: usize,
}
impl<'a, 'b> ASTIter<'a, 'b> {
    pub fn new(nodes: &'b Vec<ASTNode<'a>>) -> Self {
        Self {
            nodes,
            current_index: 0,
        }
    }

    pub fn peek(&self) -> Option<&ASTNode<'a>> {
        self.nodes.get(self.current_index)
    }

    pub fn next(&mut self) -> Option<&ASTNode<'a>> {
        let node = self.nodes.get(self.current_index);
        self.current_index += 1;
        node
    }
}

#[derive(Debug, Clone)]
pub struct ASTNode<'a> {
    pub kind: ASTNodeKind<'a>,
    pub line_number: usize,
    pub column_number: usize,
}
impl<'a> Token<'a> {
    fn into_node(&self, kind: ASTNodeKind<'a>) -> ASTNode<'a> {
        ASTNode {
            kind,
            line_number: self.line_number,
            column_number: self.column,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ASTNodeKind<'a> {
    Block(Vec<ASTNode<'a>>),
    Ident(&'a str),
    Literal(Value<'a>),
    ConfigSeparator,
}

impl<'a> ScriptAST<'a> {
    pub fn parse(mut code: TokenIterator<'a>) -> Result<Self, ParseError<'a>> {
        let mut nodes = Vec::new();

        while let Some(token) = code.next() {
            let node = match token.as_str() {
                "{" | "[" | "<" | "(" => token.into_node(ASTNodeKind::parse_block(&mut code)?),
                "\"" | "'" => token.into_node(ASTNodeKind::parse_string_literal(&mut code)?),

                _ if usize::from_str_radix(token.as_str(), 10).is_ok() => {
                    let value = usize::from_str_radix(token.as_str(), 10).unwrap();
                    token.into_node(ASTNodeKind::Literal(Value::Number(value)))
                }
                _ if token.starts_with("---") => token.into_node(ASTNodeKind::ConfigSeparator),
                _ if token.is_ascii() => token.into_node(ASTNodeKind::Ident(token.as_str())),
                _ => return Err(token.unexpected()),
            };

            //return Err(ParseError::UnexpectedToken(token));

            nodes.push(node);
        }

        Ok(Self { nodes })
    }
}

impl<'a> ASTNodeKind<'a> {
    fn parse_string_literal(code: &mut TokenIterator<'a>) -> Result<Self, ParseError<'a>> {
        let mut string = String::new();

        loop {
            let token = code.next();
            match token {
                Some(token) if token.as_str() == "\"" || token.as_str() == "'" => {
                    break;
                }
                Some(token) => {
                    if token.space_before {
                        string.push(' ');
                    }
                    string.push_str(token.as_str());
                }
                None => {
                    return Err(ParseError::UnexpectedEOF(
                        "unexpected end of string literal",
                    ))
                }
            }
        }

        Ok(Self::Literal(Value::String(string)))
    }

    fn parse_block(code: &mut TokenIterator<'a>) -> Result<Self, ParseError<'a>> {
        let mut nodes = Vec::new();

        while let Some(token) = code.next() {
            let node = match token.as_str() {
                "{" | "[" | "<" | "(" => token.into_node(ASTNodeKind::parse_block(code)?),
                "}" | "]" | ">" | ")" => return Ok(Self::Block(nodes)),
                "\"" | "'" => token.into_node(ASTNodeKind::parse_string_literal(code)?),

                _ if usize::from_str_radix(token.as_str(), 10).is_ok() => {
                    let value = usize::from_str_radix(token.as_str(), 10).unwrap();
                    token.into_node(ASTNodeKind::Literal(Value::Number(value)))
                }
                _ if token.is_ascii() => token.into_node(ASTNodeKind::Ident(token.as_str())),
                _ => return Err(token.unexpected()),
            };

            nodes.push(node);
        }

        Err(ParseError::UnexpectedEOF("block not closed"))
    }
}
