use crate::ast::*;
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
pub struct OwnedRuntimeError {
    span: String,
    pub kind: OwnedRuntimeErrorKind,
}
impl<'a> RuntimeError<'a> {
    pub fn to_owned(self) -> OwnedRuntimeError {
        OwnedRuntimeError {
            span: self.span.fragment().to_string(),
            kind: self.kind.to_owned(),
        }
    }

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

        write!(f, "{:?}", self.kind)?;

        Ok(())
    }
}

impl OwnedRuntimeError {
    pub fn borrow<'a>(&'a self) -> RuntimeError<'a> {
        RuntimeError {
            span: Span::new(self.span.as_str()),
            kind: self.kind.borrow(),
        }
    }

    pub fn to_short_string(&self) -> String {
        format!("{:?}", self.kind)
    }
}
impl core::fmt::Debug for OwnedRuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.borrow().fmt(f)
    }
}

impl core::fmt::Debug for RuntimeErrorKind<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
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
            RuntimeErrorKind::UnknownFunction(ident) => {
                writeln!(f, "call to unknown function: \"{}\"", ident)?;
            }
            RuntimeErrorKind::UnknownHook(ident) => {
                writeln!(f, "unknown hook: \"{}\"", ident)?;
            }
        };

        Ok(())
    }
}

pub enum RuntimeErrorKind<'a> {
    UndefinedVariable(&'a str),
    MissingArgument(usize, &'a str),
    InvalidOperation(Value<'a>, Value<'a>),
    InvalidConversion(Value<'a>, Value<'a>),
    UnknownFunction(&'a str),
    UnknownHook(&'a str),
}
pub enum OwnedRuntimeErrorKind {
    UndefinedVariable(String),
    MissingArgument(usize, String),
    InvalidOperation(OwnedValue, OwnedValue),
    InvalidConversion(OwnedValue, OwnedValue),
    UnknownFunction(String),
    UnknownHook(String),
}
impl RuntimeErrorKind<'_> {
    pub fn to_owned(self) -> OwnedRuntimeErrorKind {
        match self {
            RuntimeErrorKind::UndefinedVariable(ident) => {
                OwnedRuntimeErrorKind::UndefinedVariable(ident.to_string())
            }
            RuntimeErrorKind::MissingArgument(index, ident) => {
                OwnedRuntimeErrorKind::MissingArgument(index, ident.to_string())
            }
            RuntimeErrorKind::InvalidOperation(got, expected) => {
                OwnedRuntimeErrorKind::InvalidOperation(got.to_owned(), expected.to_owned())
            }
            RuntimeErrorKind::InvalidConversion(from, to) => {
                OwnedRuntimeErrorKind::InvalidConversion(from.to_owned(), to.to_owned())
            }
            RuntimeErrorKind::UnknownFunction(ident) => {
                OwnedRuntimeErrorKind::UnknownFunction(ident.to_string())
            }
            RuntimeErrorKind::UnknownHook(ident) => {
                OwnedRuntimeErrorKind::UnknownHook(ident.to_string())
            }
        }
    }
}
impl OwnedRuntimeErrorKind {
    pub fn borrow<'a>(&'a self) -> RuntimeErrorKind<'a> {
        match self {
            OwnedRuntimeErrorKind::UndefinedVariable(ident) => {
                RuntimeErrorKind::UndefinedVariable(ident)
            }
            OwnedRuntimeErrorKind::MissingArgument(index, ident) => {
                RuntimeErrorKind::MissingArgument(*index, ident.as_str())
            }
            OwnedRuntimeErrorKind::InvalidOperation(got, expected) => {
                RuntimeErrorKind::InvalidOperation(got.borrow(), expected.borrow())
            }
            OwnedRuntimeErrorKind::InvalidConversion(from, to) => {
                RuntimeErrorKind::InvalidConversion(from.borrow(), to.borrow())
            }
            OwnedRuntimeErrorKind::UnknownFunction(ident) => {
                RuntimeErrorKind::UnknownFunction(ident.as_str())
            }
            OwnedRuntimeErrorKind::UnknownHook(ident) => {
                RuntimeErrorKind::UnknownHook(ident.as_str())
            }
        }
    }
}

impl core::fmt::Debug for OwnedRuntimeErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.borrow().fmt(f)
    }
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

    pub fn from_owned(owned: &'a [(String, OwnedValue)]) -> Self {
        Self {
            root: Scope {
                variables: owned
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.borrow()))
                    .collect(),
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

    pub fn get_owned_local_scope(mut self) -> Vec<(String, OwnedValue)> {
        self.stack
            .drain(..)
            .rev()
            .take_while(|scope| !scope.is_new_scope)
            .chain(core::iter::once(self.root))
            .flat_map(|mut scope| {
                scope
                    .variables
                    .drain()
                    .map(|(k, v)| (k.to_string(), v.to_owned()))
                    .collect::<Vec<_>>()
            })
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
                    BinaryOp::Mod => lhs.modulo(&rhs),

                    BinaryOp::Eq => lhs.eq(&rhs),
                    BinaryOp::Ne => lhs
                        .eq(&rhs)
                        .and_then(|v| Ok(Value::Boolean(!v.as_boolean()?))),
                    BinaryOp::Lt => lhs.lt(&rhs),
                    BinaryOp::Gt => lhs.gt(&rhs),
                    BinaryOp::Le => lhs
                        .gt(&rhs)
                        .and_then(|v| Ok(Value::Boolean(!v.as_boolean()?))),
                    BinaryOp::Ge => lhs
                        .lt(&rhs)
                        .and_then(|v| Ok(Value::Boolean(!v.as_boolean()?))),

                    BinaryOp::And => Ok(Value::Boolean(lhs.as_boolean()? && rhs.as_boolean()?)),
                    BinaryOp::Or => Ok(Value::Boolean(lhs.as_boolean()? || rhs.as_boolean()?)),
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
                    Err(_) => interface.inbuilt_function(ident, arg_values),
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

    pub fn as_boolean(&self) -> Result<bool, RuntimeErrorKind<'a>> {
        match self {
            Value::Boolean(b) => Ok(*b),
            Value::Number(n) => Ok(*n != 0.0),
            Value::String(s) if s == "true" => Ok(true),
            Value::String(s) if s == "false" => Ok(false),
            _ => Err(RuntimeErrorKind::InvalidConversion(
                self.clone(),
                Value::Boolean(false),
            )),
        }
    }

    pub fn add(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        match (self, other) {
            // if the first value is a string, concatenate it with the second value
            (Value::String(a), other) if other.can_cast_to_string() => {
                let b = other.as_string()?;
                Ok(Value::String(format!("{}{}", a, b)))
            }

            // otherwise, try to add the values as numbers
            _ if (self.can_cast_to_number() && other.can_cast_to_number()) => {
                Ok(Value::Number(self.as_number()? + other.as_number()?))
            }

            _ => Err(RuntimeErrorKind::InvalidOperation(
                self.clone(),
                other.clone(),
            )),
        }
    }

    pub fn sub(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        Ok(Value::Number(self.as_number()? - other.as_number()?))
    }

    pub fn mul(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        Ok(Value::Number(self.as_number()? * other.as_number()?))
    }

    pub fn div(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        Ok(Value::Number(self.as_number()? / other.as_number()?))
    }

    pub fn modulo(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        Ok(Value::Number(self.as_number()? % other.as_number()?))
    }

    pub fn lt(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        Ok(Value::Boolean(self.as_number()? > other.as_number()?))
    }

    pub fn gt(&self, other: &Value<'a>) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        Ok(Value::Boolean(self.as_number()? < other.as_number()?))
    }

    fn eq(&self, other: &Self) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        match (self, other) {
            // obvious cases
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a == b)),
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a == b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a == b)),

            // number casting is usually faster (and also makes more sense) so we try that first
            _ if self.can_cast_to_number() && other.can_cast_to_number() => {
                Ok(Value::Boolean(self.as_number()? == other.as_number()?))
            }

            // last resort
            _ if self.can_cast_to_string() && other.can_cast_to_string() => {
                Ok(Value::Boolean(self.as_string()? == other.as_string()?))
            }

            _ => Ok(Value::Boolean(false)),
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
            let res = statement.run(scope, interface)?;

            if res.should_return {
                return Ok(res.to_value());
            }
        }

        Ok(Value::None)
    }
}

pub struct StatementResult<'a> {
    value: Value<'a>,
    should_return: bool,
}
impl<'a> StatementResult<'a> {
    fn new(value: Value<'a>) -> Self {
        Self {
            value,
            should_return: false,
        }
    }
    fn return_value(value: Value<'a>) -> Self {
        Self {
            value,
            should_return: true,
        }
    }

    pub fn to_value(self) -> Value<'a> {
        self.value
    }
}

impl<'a> Statement<'a> {
    pub fn run<I: Interface<'a>>(
        &self,
        scope: &mut ScopeStack<'a>,
        interface: &mut I,
    ) -> Result<StatementResult<'a>, RuntimeError<'a>> {
        let res = match &self.kind {
            StatementKind::Assignment {
                ident,
                expression,
                kind,
            } => {
                let expr_value = expression.evaluate(scope, interface).with_span(self.span)?;
                match kind {
                    AssignmentKind::Assign => scope.assign(ident, expr_value),
                    _ => {
                        let current_value = scope.get(ident).with_span(self.span)?;
                        let new_value = match kind {
                            AssignmentKind::AddAssign => {
                                current_value.add(&expr_value).with_span(self.span)?
                            }
                            AssignmentKind::SubAssign => {
                                current_value.sub(&expr_value).with_span(self.span)?
                            }
                            AssignmentKind::MulAssign => {
                                current_value.mul(&expr_value).with_span(self.span)?
                            }
                            AssignmentKind::DivAssign => {
                                current_value.div(&expr_value).with_span(self.span)?
                            }
                            AssignmentKind::Assign => unreachable!(),
                        };
                        scope.assign(ident, new_value);
                    }
                };
                StatementResult::new(Value::None)
            }
            StatementKind::Hook {
                kind,
                params,
                block,
            } => {
                let params = params
                    .iter()
                    .map(|expr| expr.evaluate(scope, interface).with_span(self.span))
                    .collect::<Result<Vec<_>, _>>()?;

                interface
                    .attach_hook(
                        kind,
                        params,
                        ScriptHook {
                            block: block.clone(),
                            local_scope: scope.get_local_scope(),
                        },
                    )
                    .with_span(self.span)?;
                StatementResult::new(Value::None)
            }
            StatementKind::Expression(expr) => {
                StatementResult::new(expr.evaluate(scope, interface).with_span(self.span)?)
            }
            StatementKind::Return { expression } => match expression {
                Some(expr) => {
                    let value = expr.evaluate(scope, interface).with_span(self.span)?;
                    StatementResult::return_value(value)
                }
                None => StatementResult::return_value(Value::None),
            },
            StatementKind::If { .. } => {
                todo!("if statement")
            }
        };

        Ok(res)
    }
}
