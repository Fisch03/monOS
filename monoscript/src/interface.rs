use crate::ast::Block;
use crate::execute::{RuntimeError, ScriptContext};

pub trait Interface<'a> {
    fn print(&self, message: &str);

    fn spawn_window(&mut self, content: WindowContent<'a>);
    fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize);
}

#[derive(Debug)]
pub struct WindowContent<'a> {
    pub(crate) block: Block<'a>,
}

impl<'a> WindowContent<'a> {
    pub fn render<I: Interface<'a>>(
        &self,
        context: &mut ScriptContext<'a>,
        interface: &mut I,
    ) -> Result<(), RuntimeError> {
        context.scope.enter_scope();
        self.block.run(&mut context.scope, interface)?;
        context.scope.exit_scope();
        Ok(())
    }
}
