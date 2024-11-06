use crate::{ast, execute::*, Interface, OwnedParseError, ParseError, ReplInterface, ScriptHook};
use alloc::{string::String, vec::Vec};

#[derive(Debug)]
pub struct ReplContext {
    pub(crate) scope: Vec<(String, ast::OwnedValue)>,
}

#[derive(Debug)]
pub enum ReplError {
    ParseError(OwnedParseError),
    RuntimeError(OwnedRuntimeError),
}
impl core::fmt::Display for ReplError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ParseError(err) => err.to_short_string().fmt(f),
            Self::RuntimeError(err) => err.to_short_string().fmt(f),
        }
    }
}
impl<'a> From<ParseError<'a>> for ReplError {
    fn from(err: ParseError<'a>) -> Self {
        Self::ParseError(err.to_owned())
    }
}
impl<'a> From<RuntimeError<'a>> for ReplError {
    fn from(err: RuntimeError<'a>) -> Self {
        Self::RuntimeError(err.to_owned())
    }
}

struct ReplInterfaceWrapper<'a, I: ReplInterface>(&'a mut I);
impl<I: ReplInterface> ReplInterface for ReplInterfaceWrapper<'_, I> {
    fn print(&mut self, message: &str) {
        self.0.print(message);
    }
}

impl<I: ReplInterface> Interface<'_> for ReplInterfaceWrapper<'_, I> {
    fn spawn_window(&mut self, _content: ScriptHook) {
        unreachable!("spawn_window not supported in repl");
    }
    fn on_key(&mut self, _key: char, _content: ScriptHook) {
        unreachable!("spawn_window not supported in repl");
    }
    fn draw_box(&mut self, _x: usize, _y: usize, _w: usize, _h: usize) {
        unreachable!("spawn_window not supported in repl");
    }
}

impl ReplContext {
    pub fn new() -> Self {
        Self { scope: Vec::new() }
    }

    pub fn execute<'a, I: ReplInterface>(
        &mut self,
        code: &'a str,
        interface: &mut I,
    ) -> Result<ast::OwnedValue, ReplError> {
        let statement = ast::Statement::parse(code.into());

        let statement = match statement {
            Ok((remainder, statement)) => {
                if remainder.fragment().is_empty() {
                    statement
                } else {
                    return Err(ParseError::unexpected_token(remainder).into());
                }
            }
            Err(err) => match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    let span = err.input;
                    return Err(ParseError::unexpected_token(span).into());
                }
                _ => unreachable!(),
            },
        };

        let mut scope = ScopeStack::from_owned(&self.scope);
        let res = statement.run(&mut scope, &mut ReplInterfaceWrapper(interface))?;
        let res = res.to_value().to_owned();
        self.scope = scope.get_owned_local_scope();

        Ok(res)
    }
}
