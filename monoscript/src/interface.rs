use crate::ast::{Block, Value};
use crate::execute::{RuntimeError, RuntimeErrorKind, ScriptContext};
use alloc::vec::Vec;

pub trait ArgArray<'a> {
    fn get_arg(&self, index: usize, ident: &'a str) -> Result<&Value<'a>, RuntimeErrorKind<'a>>;
}
impl<'a> ArgArray<'a> for &Vec<Value<'a>> {
    fn get_arg(&self, index: usize, ident: &'a str) -> Result<&Value<'a>, RuntimeErrorKind<'a>> {
        self.get(index)
            .ok_or(RuntimeErrorKind::MissingArgument(index, ident))
    }
}

impl<'a> ArgArray<'a> for Vec<Value<'a>> {
    fn get_arg(&self, index: usize, ident: &'a str) -> Result<&Value<'a>, RuntimeErrorKind<'a>> {
        self.get(index)
            .ok_or(RuntimeErrorKind::MissingArgument(index, ident))
    }
}

pub trait Interface<'a> {
    fn inbuilt_function<A: ArgArray<'a>>(
        &mut self,
        ident: &'a str,
        args: A,
    ) -> Result<Value<'a>, RuntimeErrorKind<'a>>;

    fn attach_hook<A: ArgArray<'a>>(
        &mut self,
        kind: &'a str,
        params: A,
        hook: ScriptHook<'a>,
    ) -> Result<(), RuntimeErrorKind<'a>>;
}

#[derive(Debug)]
pub struct ScriptHook<'a> {
    pub(crate) block: Block<'a>,
    pub(crate) local_scope: Vec<(&'a str, Value<'a>)>,
}

impl<'a> ScriptHook<'a> {
    pub fn execute<I: Interface<'a>>(
        &self,
        context: &mut ScriptContext<'a>,
        interface: &mut I,
    ) -> Result<(), RuntimeError> {
        context.scope.enter_scope(self.local_scope.clone());
        self.block.run(&mut context.scope, interface)?;
        context.scope.exit_scope();
        Ok(())
    }
}
