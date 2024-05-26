use crate::{
    interface::PersistentCode,
    interpret::{CodeBlock, Expression, Script, Statement, Value, Window},
    Interface,
};

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::HashMap;

#[derive(Debug)]
pub enum RuntimeError<'a> {
    UndefinedVariable(&'a str),
    MissingArgument(&'a str),
    MismatchedTypes(Value<'a>, Value<'a>),
}

pub struct ScriptContext<'a> {
    pub(crate) script: Script<'a>,
    pub(crate) scope: ScopeStack<'a>,
}
pub struct ScopeStack<'a> {
    root: Scope<'a>,
    stack: Vec<Scope<'a>>,
}
impl<'a> ScopeStack<'a> {
    pub fn new() -> Self {
        Self {
            root: Scope {
                variables: HashMap::new(),
            },
            stack: Vec::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.stack.push(Scope {
            variables: HashMap::new(),
        });
    }
    pub fn exit_scope(&mut self) {
        self.stack.pop();
    }

    #[inline]
    fn current_scope_mut(&mut self) -> &mut Scope<'a> {
        self.stack.last_mut().unwrap_or(&mut self.root)
    }

    pub fn assign(&mut self, ident: &'a str, value: Value<'a>) {
        self.current_scope_mut().variables.insert(ident, value);
    }

    pub fn get(&self, ident: &'a str) -> Result<&Value<'a>, RuntimeError<'a>> {
        self.stack
            .iter()
            .rev()
            .find_map(|scope| scope.variables.get(ident))
            .or_else(|| self.root.variables.get(ident))
            .ok_or(RuntimeError::UndefinedVariable(ident))
    }
}

pub struct Scope<'a> {
    variables: HashMap<&'a str, Value<'a>>,
}

impl<'a> ScriptContext<'a> {
    pub fn new(script: Script<'a>) -> Self {
        Self {
            script,
            scope: ScopeStack::new(),
        }
    }

    pub fn run<I: Interface>(
        mut self,
        interface: &mut I,
    ) -> Result<Option<PersistentCode<'a>>, RuntimeError<'a>> {
        self.script.code.run(&mut self.scope, interface)?;

        if self.script.window.is_some() {
            Ok(Some(PersistentCode::new(self)))
        } else {
            Ok(None)
        }
    }
}

impl<'a> Expression<'a> {
    pub fn evaluate(&self, scope: &ScopeStack<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        match self {
            Expression::Literal(value) => Ok(value.clone()),
            Expression::Ident(ident) => Ok(scope.get(ident).cloned()?),
        }
    }
}

impl Value<'_> {
    pub fn print_value(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::None => "None".to_string(),
            Value::Function { .. } => "Function".into(),
            Value::Boolean(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
        }
    }

    pub fn as_number(&self) -> Option<usize> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }
}

impl<'a> Window<'a> {
    pub fn render<I: Interface>(
        &self,
        scope: &mut ScopeStack<'a>,
        interface: &mut I,
    ) -> Result<(), RuntimeError> {
        scope.enter_scope();
        self.render.run(scope, interface)?;
        scope.exit_scope();
        Ok(())
    }
}

impl<'a> CodeBlock<'a> {
    fn run<I: Interface>(
        &self,
        scope: &mut ScopeStack<'a>,
        interface: &mut I,
    ) -> Result<(), RuntimeError<'a>> {
        for statement in self.iter() {
            match statement {
                Statement::Assignment(ident, expression) => {
                    let value = expression.evaluate(&scope)?;
                    scope.assign(ident, value);
                }
                Statement::FunctionCall(ident, args) => {
                    let arg_values = args
                        .iter()
                        .map(|arg| arg.evaluate(&scope))
                        .collect::<Result<Vec<_>, _>>()?;

                    let function = scope.get(ident).cloned();
                    match function {
                        Ok(Value::Function { args, body }) => {
                            scope.enter_scope();
                            for (arg, value) in args.iter().zip(arg_values) {
                                scope.assign(arg, value);
                            }
                            body.run(scope, interface)?; // TODO: function return value
                            scope.exit_scope();
                        }
                        Ok(_) => return Err(RuntimeError::UndefinedVariable(ident)),
                        Err(_) => {
                            inbuilt_function(ident, arg_values, interface)?;
                        }
                    }
                }
                _ => todo!(),
            };
        }

        Ok(())
    }
}

fn inbuilt_function<'a, I: Interface>(
    ident: &'a str,
    args: Vec<Value<'a>>,
    interface: &mut I,
) -> Result<Value<'a>, RuntimeError<'a>> {
    match ident {
        "print" => {
            for arg in args {
                interface.print(&arg.print_value());
            }
            interface.print("\n");
            Ok(Value::None)
        }
        "debug" => {
            for arg in args {
                interface.print(&format!("{:?}", arg));
            }
            interface.print("\n");
            Ok(Value::None)
        }

        "box" => {
            let x = args
                .get(0)
                .ok_or(RuntimeError::MissingArgument("box x coordinate"))?;
            let y = args
                .get(1)
                .ok_or(RuntimeError::MissingArgument("box y coordinate"))?;
            let w = args
                .get(2)
                .ok_or(RuntimeError::MissingArgument("box width"))?;
            let h = args
                .get(3)
                .ok_or(RuntimeError::MissingArgument("box height"))?;

            interface.draw_box(
                x.as_number()
                    .ok_or(RuntimeError::MismatchedTypes(x.clone(), Value::Number(0)))?,
                y.as_number()
                    .ok_or(RuntimeError::MismatchedTypes(y.clone(), Value::Number(0)))?,
                w.as_number()
                    .ok_or(RuntimeError::MismatchedTypes(w.clone(), Value::Number(0)))?,
                h.as_number()
                    .ok_or(RuntimeError::MismatchedTypes(h.clone(), Value::Number(0)))?,
            );

            Ok(Value::None)
        }
        "square" => {
            let x = args
                .get(0)
                .ok_or(RuntimeError::MissingArgument("square x coordinate"))?;
            let y = args
                .get(1)
                .ok_or(RuntimeError::MissingArgument("square y coordinate"))?;
            let s = args
                .get(2)
                .ok_or(RuntimeError::MissingArgument("square size"))?;

            interface.draw_box(
                x.as_number()
                    .ok_or(RuntimeError::MismatchedTypes(x.clone(), Value::Number(0)))?,
                y.as_number()
                    .ok_or(RuntimeError::MismatchedTypes(y.clone(), Value::Number(0)))?,
                s.as_number()
                    .ok_or(RuntimeError::MismatchedTypes(s.clone(), Value::Number(0)))?,
                s.as_number()
                    .ok_or(RuntimeError::MismatchedTypes(s.clone(), Value::Number(0)))?,
            );

            Ok(Value::None)
        }
        _ => Err(RuntimeError::UndefinedVariable(ident)),
    }
}
