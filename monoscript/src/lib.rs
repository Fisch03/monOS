#![no_std]

extern crate alloc;
use core::str;

pub mod ast;
use ast::{ParseError, ScriptAST};

pub mod interpret;
use interface::PersistentCode;
use interpret::{InterpretError, Script};

pub mod execute;
use execute::{RuntimeError, ScriptContext};

pub mod interface;
pub use interface::Interface;

pub fn parse<'a>(code: &'a str) -> Result<ScriptAST<'a>, ParseError<'a>> {
    let code = TokenIterator::new(code);
    ScriptAST::parse(code)
}

pub fn interpret<'a>(ast: ScriptAST<'a>) -> Result<Script<'a>, InterpretError<'a>> {
    Script::interpret(ast)
}

pub fn execute<'a, I: Interface>(
    script: Script<'a>,
    interface: &mut I,
) -> Result<Option<PersistentCode<'a>>, RuntimeError<'a>> {
    let context = ScriptContext::new(script);
    context.run(interface)
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
