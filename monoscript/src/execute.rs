use crate::ast::{
    AssignmentKind, BinaryOp, Block, Expression, HookType, Span, StatementKind, UnaryOp, Value,
};
use crate::{Interface, Script, ScriptHook};

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::HashMap;

trait AddSpanError<'a> {
    type Output;
    fn with_span(self, span: Span<'a>) -> Self::Output;
}

pub struct RuntimeError<'a> {
    span: Span<'a>,
    pub kind: RuntimeErrorKind<'a>,
}
impl<'a> RuntimeError<'a> {
    fn current_line(&self) -> &str {
        let remainder = self.span.fragment();
        remainder.lines().next().unwrap_or("")
    }
}
impl core::fmt::Debug for RuntimeError<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "in line: {}, column: {}",
            self.span.location_line(),
            self.span.location_offset()
        )?;
        writeln!(f, "at \"{}\"", self.current_line())?;

        match &self.kind {
            RuntimeErrorKind::UndefinedVariable(ident) => {
                writeln!(f, "tried to access undefined variable: \"{}\"", ident)?;
            }
            RuntimeErrorKind::MissingArgument(index, ident) => {
                writeln!(
                    f,
                    "missing argument no. {} for function call: {}",
                    index + 1,
                    ident
                )?;
            }
            RuntimeErrorKind::InvalidOperation(got, expected) => {
                writeln!(
                    f,
                    "cannot perform operation on {} and {}",
                    expected.print_type(),
                    got.print_type()
                )?;
            }
            RuntimeErrorKind::InvalidConversion(from, to) => {
                writeln!(
                    f,
                    "invalid conversion from {} to {}",
                    from.print_type(),
                    to.print_type()
                )?;
            }
        }

        Ok(())
    }
}
pub enum RuntimeErrorKind<'a> {
    UndefinedVariable(&'a str),
    MissingArgument(usize, &'a str),
    InvalidOperation(Value<'a>, Value<'a>),
    InvalidConversion(Value<'a>, Value<'a>),
}

#[derive(Debug)]
pub struct ScriptContext<'a> {
    pub(crate) script: Script<'a>,
    pub(crate) scope: ScopeStack<'a>,
}
impl<'a> AddSpanError<'a> for RuntimeErrorKind<'a> {
    type Output = RuntimeError<'a>;
    fn with_span(self, span: Span<'a>) -> Self::Output {
        RuntimeError { span, kind: self }
    }
}
impl<'a, T> AddSpanError<'a> for Result<T, RuntimeErrorKind<'a>> {
    type Output = Result<T, RuntimeError<'a>>;
    fn with_span(self, span: Span<'a>) -> Self::Output {
        self.map_err(move |kind| RuntimeError { span, kind })
    }
}

#[derive(Debug)]
pub struct ScopeStack<'a> {
    root: Scope<'a>,
    stack: Vec<Scope<'a>>,
}
impl<'a> ScopeStack<'a> {
    pub fn new() -> Self {
        Self {
            root: Scope {
                variables: HashMap::with_hasher(rustc_hash::FxBuildHasher::default()),
                is_new_scope: false,
            },
            stack: Vec::new(),
        }
    }

    pub fn enter_block(&mut self) {
        self.stack.push(Scope {
            variables: HashMap::with_hasher(rustc_hash::FxBuildHasher::default()),
            is_new_scope: false,
        });
    }
    pub fn enter_scope(&mut self, scope: Vec<(&'a str, Value<'a>)>) {
        self.stack.push(Scope {
            variables: scope.into_iter().collect(),
            is_new_scope: false,
        });
    }
    pub fn enter_function(&mut self, arg_names: Vec<&'a str>, arg_values: Vec<Value<'a>>) {
        self.stack.last_mut().map(|scope| scope.is_new_scope = true);
        self.stack.push(Scope {
            variables: arg_names.into_iter().zip(arg_values).collect(),
            is_new_scope: false,
        });
    }
    pub fn exit_scope(&mut self) {
        self.stack.pop();
        self.stack
            .last_mut()
            .map(|scope| scope.is_new_scope = false);
    }

    #[inline]
    fn current_scope_mut(&mut self) -> &mut Scope<'a> {
        self.stack.last_mut().unwrap_or(&mut self.root)
    }

    pub fn assign(&mut self, ident: &'a str, value: Value<'a>) {
        use hashbrown::hash_map::Entry;

        let mut found = false;
        for scope in self
            .stack
            .iter_mut()
            .rev()
            .take_while(|scope| !scope.is_new_scope)
            .chain(core::iter::once(&mut self.root))
        {
            match scope.variables.entry(ident) {
                Entry::Vacant(_) => false,
                Entry::Occupied(mut entry) => {
                    entry.insert(value.clone());
                    found = true;
                    break;
                }
            };
        }

        if !found {
            self.current_scope_mut().variables.insert(ident, value);
        }
    }

    pub fn get(&self, ident: &'a str) -> Result<&Value<'a>, RuntimeErrorKind<'a>> {
        self.stack
            .iter()
            .rev()
            .take_while(|scope| !scope.is_new_scope)
            .chain(core::iter::once(&self.root))
            .find_map(|scope| scope.variables.get(ident))
            .ok_or(RuntimeErrorKind::UndefinedVariable(ident))
    }

    pub fn get_local_scope(&self) -> Vec<(&'a str, Value<'a>)> {
        self.stack
            .iter()
            .rev()
            .take_while(|scope| !scope.is_new_scope)
            .flat_map(|scope| scope.variables.iter().map(|(k, v)| (*k, v.clone())))
            .collect()
    }
}

#[derive(Debug)]
pub struct Scope<'a> {
    variables: HashMap<&'a str, Value<'a>, rustc_hash::FxBuildHasher>,
    is_new_scope: bool,
}

impl<'a> ScriptContext<'a> {
    pub fn new(script: Script<'a>) -> Self {
        Self {
            script,
            scope: ScopeStack::new(),
        }
    }

    pub fn run<I: Interface<'a>>(&mut self, interface: &mut I) -> Result<Value, RuntimeError<'a>> {
        self.script.0.run(&mut self.scope, interface)
    }
}

impl<'a> Expression<'a> {
    pub fn evaluate<I: Interface<'a>>(
        &self,
        scope: &mut ScopeStack<'a>,
        interface: &mut I,
    ) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        match self {
            Expression::Literal(value) => Ok(value.clone()),
            Expression::Identifier(ident) => Ok(scope.get(ident).cloned()?),
            Expression::Unary { op, expr } => {
                let value = expr.evaluate(scope, interface)?;
                match op {
                    UnaryOp::Neg => match value {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(RuntimeErrorKind::InvalidOperation(
                            value.clone(),
                            Value::Number(0.0),
                        )),
                    },
                    UnaryOp::Not => match value {
                        Value::Boolean(b) => Ok(Value::Boolean(!b)),
                        _ => Err(RuntimeErrorKind::InvalidOperation(
                            value.clone(),
                            Value::Boolean(false),
                        )),
                    },
                }
            }
            Expression::Binary { op, lhs, rhs } => {
                let lhs = lhs.evaluate(scope, interface)?;
                let rhs = rhs.evaluate(scope, interface)?;
                match op {
                    BinaryOp::Add => lhs.add(&rhs),
                    BinaryOp::Sub => lhs.sub(&rhs),
                    BinaryOp::Mul => lhs.mul(&rhs),
                    BinaryOp::Div => lhs.div(&rhs),
                    _ => todo!("BinaryOp::{:?} not implemented", op),
                }
            }
            Expression::FunctionCall { ident, args } => {
                let arg_values = args
                    .iter()
                    .map(|arg| arg.evaluate(scope, interface))
                    .collect::<Result<Vec<_>, _>>()?;

                let function = scope.get(ident).cloned();
                match function {
                    Ok(Value::Function { args, block }) => {
                        scope.enter_function(args, arg_values);
                        let ret = block.run(scope, interface);
                        scope.exit_scope();
                        ret.map_err(|err| err.kind)
                    }
                    Ok(_) => Err(RuntimeErrorKind::UndefinedVariable(ident)),
                    Err(_) => inbuilt_function(ident, arg_values, interface),
                }
            }
        }
    }
}

impl<'a> Value<'a> {
    pub fn print_type(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Function { .. } => "function",
            Value::None => "none",
        }
    }

    // print the value as a string, for debugging purposes
    pub fn print_value(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Function { .. } => "Function".into(),
            Value::Boolean(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Value::None => "None".into(),
        }
    }

    // cast the value into a number, if possible
    pub fn can_cast_to_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            Value::String(s) => s.parse::<f64>().is_ok(),
            Value::Boolean(_) => true,
            _ => false,
        }
    }
    pub fn as_number(&self) -> Result<f64, RuntimeErrorKind<'a>> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::String(s) => s
                .parse()
                .map_err(|_| RuntimeErrorKind::InvalidConversion(self.clone(), Value::Number(0.0))),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(RuntimeErrorKind::InvalidConversion(
                self.clone(),
                Value::Number(0.0),
            )),
        }
    }

    // cast the value into a string, if possible
    pub fn can_cast_to_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            Value::Number(_) => true,
            Value::Boolean(_) => true,
            _ => false,
        }
    }
    pub fn as_string(&self) -> Result<String, RuntimeErrorKind<'a>> {
        match self {
            Value::String(s) => Ok(s.to_string()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Boolean(b) => Ok(if *b { "true".into() } else { "false".into() }),
            _ => Result::Err(RuntimeErrorKind::InvalidConversion(
                self.clone(),
                Value::String("".into()),
            )),
        }
    }

    pub fn add(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        match (self, other) {
            (Value::Number(a), other) if other.can_cast_to_number() => {
                Ok(Value::Number(a + other.as_number()?))
            }
            (other, Value::Number(b)) if other.can_cast_to_number() => {
                Ok(Value::Number(other.as_number()? + b))
            }

            (Value::String(a), other) if other.can_cast_to_string() => {
                let b = other.as_string()?;
                Ok(Value::String(format!("{}{}", a, b)))
            }
            (other, Value::String(b)) if other.can_cast_to_string() => {
                let a = other.as_string()?;
                Ok(Value::String(format!("{}{}", a, b)))
            }

            _ => Err(RuntimeErrorKind::InvalidOperation(
                self.clone(),
                other.clone(),
            )),
        }
    }

    pub fn sub(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        match (self, other) {
            (Value::Number(a), other) if other.can_cast_to_number() => {
                Ok(Value::Number(a - other.as_number()?))
            }
            (other, Value::Number(b)) if other.can_cast_to_number() => {
                Ok(Value::Number(other.as_number()? - b))
            }

            _ => Err(RuntimeErrorKind::InvalidOperation(
                self.clone(),
                other.clone(),
            )),
        }
    }

    pub fn mul(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        match (self, other) {
            (Value::Number(a), other) if other.can_cast_to_number() => {
                Ok(Value::Number(a * other.as_number()?))
            }
            (other, Value::Number(b)) if other.can_cast_to_number() => {
                Ok(Value::Number(other.as_number()? * b))
            }

            _ => Err(RuntimeErrorKind::InvalidOperation(
                self.clone(),
                other.clone(),
            )),
        }
    }

    pub fn div(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        match (self, other) {
            (Value::Number(a), other) if other.can_cast_to_number() => {
                Ok(Value::Number(a / other.as_number()?))
            }
            (other, Value::Number(b)) if other.can_cast_to_number() => {
                Ok(Value::Number(other.as_number()? / b))
            }
            _ => Err(RuntimeErrorKind::InvalidOperation(
                self.clone(),
                other.clone(),
            )),
        }
    }
}

impl<'a> Block<'a> {
    pub fn run<I: Interface<'a>>(
        &self,
        scope: &mut ScopeStack<'a>,
        interface: &mut I,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        for statement in self.statements.iter() {
            match &statement.kind {
                StatementKind::Assignment {
                    ident,
                    expression,
                    kind,
                } => {
                    let expr_value = expression
                        .evaluate(scope, interface)
                        .with_span(statement.span)?;
                    match kind {
                        AssignmentKind::Assign => scope.assign(ident, expr_value),
                        _ => {
                            let current_value = scope.get(ident).with_span(statement.span)?;
                            let new_value = match kind {
                                AssignmentKind::AddAssign => {
                                    current_value.add(&expr_value).with_span(statement.span)?
                                }
                                AssignmentKind::SubAssign => {
                                    current_value.sub(&expr_value).with_span(statement.span)?
                                }
                                AssignmentKind::MulAssign => {
                                    current_value.mul(&expr_value).with_span(statement.span)?
                                }
                                AssignmentKind::DivAssign => {
                                    current_value.div(&expr_value).with_span(statement.span)?
                                }
                                AssignmentKind::Assign => unreachable!(),
                            };
                            scope.assign(ident, new_value);
                        }
                    };
                }
                StatementKind::Hook { kind, block } => match kind {
                    HookType::Window => {
                        interface.spawn_window(ScriptHook {
                            block: block.clone(),
                            local_scope: scope.get_local_scope(),
                        });
                    }
                    HookType::Key(char) => interface.on_key(
                        *char,
                        ScriptHook {
                            block: block.clone(),
                            local_scope: scope.get_local_scope(),
                        },
                    ),
                },
                StatementKind::Expression(expr) => {
                    expr.evaluate(scope, interface).with_span(statement.span)?;
                }
                StatementKind::Return { expression } => match expression {
                    Some(expr) => {
                        let value = expr.evaluate(scope, interface).with_span(statement.span)?;
                        return Ok(value);
                    }
                    None => return Ok(Value::None),
                },
                StatementKind::If { .. } => {
                    todo!("if statement")
                }
            };
        }

        Ok(Value::None)
    }
}

trait ArgArray<'a> {
    fn get_arg(&self, index: usize, ident: &'a str) -> Result<&Value<'a>, RuntimeErrorKind<'a>>;
}
impl<'a> ArgArray<'a> for Vec<Value<'a>> {
    fn get_arg(&self, index: usize, ident: &'a str) -> Result<&Value<'a>, RuntimeErrorKind<'a>> {
        self.get(index)
            .ok_or(RuntimeErrorKind::MissingArgument(index, ident))
    }
}

fn inbuilt_function<'a, I: Interface<'a>>(
    ident: &'a str,
    args: Vec<Value<'a>>,
    interface: &mut I,
) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
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
            interface.draw_box(
                args.get_arg(0, "box x position")?.as_number()? as usize,
                args.get_arg(1, "box y position")?.as_number()? as usize,
                args.get_arg(2, "box width")?.as_number()? as usize,
                args.get_arg(3, "box height")?.as_number()? as usize,
            );

            Ok(Value::None)
        }
        "square" => {
            interface.draw_box(
                args.get_arg(0, "square x position")?.as_number()? as usize,
                args.get_arg(1, "square y position")?.as_number()? as usize,
                args.get_arg(2, "square size")?.as_number()? as usize,
                args.get_arg(2, "square size")?.as_number()? as usize,
            );

            Ok(Value::None)
        }
        _ => Err(RuntimeErrorKind::UndefinedVariable(ident)),
    }
}
