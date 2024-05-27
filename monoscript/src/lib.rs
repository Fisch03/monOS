// #![no_std]

extern crate alloc;
use core::str;

pub mod ast;
use ast::{Block, Span};

pub mod parse;

pub mod execute;
use execute::RuntimeError;
pub use execute::ScriptContext;

pub mod interface;
pub use interface::{Interface, WindowContent};

#[derive(Debug)]
pub enum ParseError<'a> {
    UnexpectedToken(Span<'a>),
}

use nom::IResult;
#[derive(Debug)]
pub struct Script<'a>(Block<'a>);
impl<'a> TryFrom<IResult<Span<'a>, Block<'a>>> for Script<'a> {
    type Error = ParseError<'a>;
    fn try_from(result: IResult<Span<'a>, Block<'a>>) -> Result<Self, Self::Error> {
        match result {
            Ok((remainder, block)) => {
                if remainder.fragment().is_empty() {
                    Ok(Script(block))
                } else {
                    Err(ParseError::UnexpectedToken(remainder))
                }
            }
            Err(err) => match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    let span = err.input;
                    Err(ParseError::UnexpectedToken(span))
                }
                _ => unreachable!(),
            },
        }
    }
}

pub fn parse<'a>(code: &'a str) -> Result<Script<'a>, ParseError<'a>> {
    let code = ast::Block::parse(code.into());
    Script::try_from(code)
}

pub fn execute<'a, I: Interface<'a>>(
    script: Script<'a>,
    interface: &mut I,
) -> Result<ScriptContext<'a>, RuntimeError<'a>> {
    let mut context = ScriptContext::new(script);
    context.run(interface)?;
    Ok(context)
}

#[derive(Debug)]
pub struct TokenIterator<'a> {
    code: &'a str,
    current_index: usize,
    line_number: usize,
    column_number: usize,
}
impl<'a> TokenIterator<'a> {
    pub fn new(code: &'a str) -> TokenIterator<'a> {
        Self {
            code,
            line_number: 0,
            current_index: 0,
            column_number: 0,
        }
    }

    #[inline]
    fn ends_token(&mut self, c: char) -> bool {
        if c.is_whitespace() || (!c.is_alphanumeric() && c != '-') {
            true
        } else {
            false
        }
    }
}
impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let chars = &self.code[self.current_index..];

        let mut iter = chars.char_indices();

        let start;
        loop {
            match iter.next() {
                Some((i, c)) => {
                    self.column_number += 1;
                    if c == '\n' {
                        self.line_number += 1;
                        self.column_number = 0;
                    }
                    if !c.is_whitespace() {
                        start = self.current_index + i;
                        break;
                    }
                }
                None => return None,
            }
        }

        let start_column = self.column_number;
        let mut end = start;
        if !self.ends_token(self.code[start..].chars().next().unwrap()) {
            loop {
                match iter.next() {
                    Some((i, c)) => {
                        self.column_number += 1;
                        if c == '\n' {
                            self.line_number += 1;
                            self.column_number = 0;
                        }
                        if self.ends_token(c) {
                            end = self.current_index + i - 1;
                            break;
                        }
                    }
                    None => break,
                }
            }
        }

        let token = Token {
            value: &self.code[start..=end].trim(),
            line_number: self.line_number,
            column: start_column,
            space_before: start != self.current_index,
        };

        self.current_index = end + 1;

        Some(token)
    }
}

#[derive(Debug)]
pub struct Token<'a> {
    pub value: &'a str,
    pub line_number: usize,
    pub column: usize,
    pub space_before: bool,
}
impl<'a> Token<'a> {
    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.value
    }
}

impl core::ops::Deref for Token<'_> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}
