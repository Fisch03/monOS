use crate::execute::ScriptContext;

pub trait Interface {
    fn print(&self, message: &str);

    fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize);
}

#[must_use]
pub struct PersistentCode<'a> {
    context: ScriptContext<'a>,
}

impl<'a> PersistentCode<'a> {
    pub(crate) fn new(context: ScriptContext<'a>) -> Self {
        Self { context }
    }

    #[inline]
    pub fn wants_window(&self) -> bool {
        self.context.script.window.is_some()
    }

    #[inline]
    pub fn on_window<I: Interface>(&mut self, interface: &mut I) {
        if let Some(window) = &self.context.script.window {
            window.render(&mut self.context.scope, interface).unwrap();
        }
    }
}
