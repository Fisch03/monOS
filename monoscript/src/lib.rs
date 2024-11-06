#![no_std]

extern crate alloc;
use alloc::{
    format,
    string::{String, ToString},
};
use core::str;

pub mod ast;
use ast::{Block, Span};

pub mod parse;

pub mod execute;
use execute::RuntimeError;
pub use execute::ScriptContext;

pub mod interface;
pub use interface::{Interface, ReplInterface, ScriptHook};

pub mod repl;
pub use repl::ReplContext;

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

use nom::IResult;

pub struct ParseError<'a> {
    span: Span<'a>,
    kind: ParseErrorKind,
}
pub struct OwnedParseError {
    span: String,
    kind: ParseErrorKind,
}
#[derive(Clone, Copy)]
pub enum ParseErrorKind {
    UnexpectedToken,
}

impl OwnedParseError {
    pub fn to_short_string(&self) -> String {
        match self.kind {
            ParseErrorKind::UnexpectedToken => format!("unexpected token at \"{}\"", self.span),
        }
    }
}

impl<'a> ParseError<'a> {
    #[inline]
    pub fn unexpected_token(span: Span<'a>) -> ParseError<'a> {
        Self {
            span,
            kind: ParseErrorKind::UnexpectedToken,
        }
    }

    fn current_line(&self) -> &str {
        let remainder = self.span.fragment();
        remainder.lines().next().unwrap_or("")
    }

    pub fn to_owned(&self) -> OwnedParseError {
        OwnedParseError {
            span: self.span.fragment().to_string(),
            kind: self.kind,
        }
    }
}

impl core::fmt::Debug for ParseError<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "in line: {}, column: {}",
            self.span.location_line(),
            self.span.location_offset()
        )?;
        writeln!(f, "at \"{}\"", self.current_line())?;

        match self.kind {
            ParseErrorKind::UnexpectedToken => {
                writeln!(f, "unexpected token")?;
            }
        }

        Ok(())
    }
}
impl core::fmt::Debug for OwnedParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "at \"{}\"", self.span)?;
        match self.kind {
            ParseErrorKind::UnexpectedToken => {
                writeln!(f, "unexpected token")?;
            }
        }
        Ok(())
    }
}

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
                    Err(ParseError::unexpected_token(remainder))
                }
            }
            Err(err) => match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    let span = err.input;
                    Err(ParseError::unexpected_token(span))
                }
                _ => unreachable!(),
            },
        }
    }
}
