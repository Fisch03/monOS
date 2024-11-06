use crate::ast::{Block, Value};
use crate::execute::{RuntimeError, ScriptContext};
use alloc::vec::Vec;

pub trait Interface<'a>: ReplInterface {
    fn spawn_window(&mut self, content: ScriptHook<'a>);
    fn on_key(&mut self, key: char, content: ScriptHook<'a>);

    fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize);
}

pub trait ReplInterface {
    fn print(&mut self, message: &str);
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
