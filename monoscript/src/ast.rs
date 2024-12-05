use nom_locate::LocatedSpan;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

/// operator affecting a single operand
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// operator affecting two operands
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinaryOp {
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Eq | BinaryOp::Ne => 3,
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => 4,
            BinaryOp::Add | BinaryOp::Sub => 5,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 6,
        }
    }
}

/// ways to assign a value to a variable
#[derive(Debug, Clone, Copy)]
pub enum AssignmentKind {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

pub type Span<'a> = LocatedSpan<&'a str>;

/// a list of statements (think: function body, etc)
#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub span: Span<'a>,
    pub statements: Vec<Statement<'a>>,
}
#[derive(Debug, Clone)]
pub struct OwnedBlock {
    pub statements: Vec<OwnedStatement>,
}
impl Block<'_> {
    pub fn to_owned(self) -> OwnedBlock {
        OwnedBlock {
            statements: self
                .statements
                .into_iter()
                .map(|stmt| stmt.to_owned())
                .collect(),
        }
    }
}
impl OwnedBlock {
    pub fn borrow(&self) -> Block {
        Block {
            statements: self.statements.iter().map(|stmt| stmt.borrow()).collect(),
            span: Span::new("<unknown>"),
        }
    }
}

/// a single statement (think: a line of code)
#[derive(Debug, Clone)]
pub struct Statement<'a> {
    pub span: Span<'a>,
    pub kind: StatementKind<'a>,
}
#[derive(Debug, Clone)]
pub struct OwnedStatement {
    pub kind: OwnedStatementKind,
}
impl Statement<'_> {
    pub fn to_owned(self) -> OwnedStatement {
        OwnedStatement {
            kind: self.kind.to_owned(),
        }
    }
}
impl OwnedStatement {
    pub fn borrow(&self) -> Statement {
        Statement {
            kind: self.kind.borrow(),
            span: Span::new("<unknown>"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StatementKind<'a> {
    Assignment {
        ident: &'a str,
        expression: Expression<'a>,
        kind: AssignmentKind,
    },
    Hook {
        kind: &'a str,
        params: Vec<Expression<'a>>,
        block: Block<'a>,
    },
    If {
        condition: Expression<'a>,
        block: Block<'a>,
        else_block: Option<Block<'a>>,
    },
    Return {
        expression: Option<Expression<'a>>,
    },
    Expression(Expression<'a>),
}
#[derive(Debug, Clone)]
pub enum OwnedStatementKind {
    Assignment {
        ident: String,
        expression: OwnedExpression,
        kind: AssignmentKind,
    },
    Hook {
        kind: String,
        params: Vec<OwnedExpression>,
        block: OwnedBlock,
    },
    If {
        condition: OwnedExpression,
        block: OwnedBlock,
        else_block: Option<OwnedBlock>,
    },
    Return {
        expression: Option<OwnedExpression>,
    },
    Expression(OwnedExpression),
}
impl StatementKind<'_> {
    pub fn to_owned(self) -> OwnedStatementKind {
        match self {
            StatementKind::Assignment {
                ident,
                expression,
                kind,
            } => OwnedStatementKind::Assignment {
                ident: ident.to_string(),
                expression: expression.to_owned(),
                kind,
            },
            StatementKind::Hook {
                kind: name,
                params,
                block,
            } => OwnedStatementKind::Hook {
                kind: name.to_string(),
                params: params.into_iter().map(|param| param.to_owned()).collect(),
                block: block.to_owned(),
            },
            StatementKind::If {
                condition,
                block,
                else_block,
            } => OwnedStatementKind::If {
                condition: condition.to_owned(),
                block: block.to_owned(),
                else_block: else_block.map(|block| block.to_owned()),
            },
            StatementKind::Return { expression } => OwnedStatementKind::Return {
                expression: expression.map(|expr| expr.to_owned()),
            },
            StatementKind::Expression(expr) => OwnedStatementKind::Expression(expr.to_owned()),
        }
    }
}
impl OwnedStatementKind {
    pub fn borrow<'a>(&'a self) -> StatementKind<'a> {
        match self {
            OwnedStatementKind::Assignment {
                ident,
                expression,
                kind,
            } => StatementKind::Assignment {
                ident: ident.as_str(),
                expression: expression.borrow(),
                kind: *kind,
            },
            OwnedStatementKind::Hook {
                kind,
                params,
                block,
            } => StatementKind::Hook {
                kind: kind.as_str(),
                params: params.iter().map(|param| param.borrow()).collect(),
                block: block.borrow(),
            },
            OwnedStatementKind::If {
                condition,
                block,
                else_block,
            } => StatementKind::If {
                condition: condition.borrow(),
                block: block.borrow(),
                else_block: else_block.as_ref().map(|block| block.borrow()),
            },
            OwnedStatementKind::Return { expression } => StatementKind::Return {
                expression: expression.as_ref().map(|expr| expr.borrow()),
            },
            OwnedStatementKind::Expression(expr) => StatementKind::Expression(expr.borrow()),
        }
    }
}

/// an expression is a piece of code that evaluates to a value
#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Literal(Value<'a>),
    Identifier(&'a str),
    Unary {
        op: UnaryOp,
        expr: Box<Expression<'a>>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expression<'a>>,
        rhs: Box<Expression<'a>>,
    },
    FunctionCall {
        ident: &'a str,
        args: Vec<Expression<'a>>,
    },
}
#[derive(Debug, Clone)]
pub enum OwnedExpression {
    Literal(OwnedValue),
    Identifier(String),
    Unary {
        op: UnaryOp,
        expr: Box<OwnedExpression>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<OwnedExpression>,
        rhs: Box<OwnedExpression>,
    },
    FunctionCall {
        ident: String,
        args: Vec<OwnedExpression>,
    },
}
impl Expression<'_> {
    pub fn to_owned(self) -> OwnedExpression {
        match self {
            Expression::Literal(value) => OwnedExpression::Literal(value.to_owned()),
            Expression::Identifier(ident) => OwnedExpression::Identifier(ident.to_string()),
            Expression::Unary { op, expr } => OwnedExpression::Unary {
                op,
                expr: Box::new(expr.to_owned()),
            },
            Expression::Binary { op, lhs, rhs } => OwnedExpression::Binary {
                op,
                lhs: Box::new(lhs.to_owned()),
                rhs: Box::new(rhs.to_owned()),
            },
            Expression::FunctionCall { ident, args } => OwnedExpression::FunctionCall {
                ident: ident.to_string(),
                args: args.into_iter().map(|arg| arg.to_owned()).collect(),
            },
        }
    }
}
impl OwnedExpression {
    pub fn borrow(&self) -> Expression {
        match self {
            OwnedExpression::Literal(value) => Expression::Literal(value.borrow()),
            OwnedExpression::Identifier(ident) => Expression::Identifier(ident.as_str()),
            OwnedExpression::Unary { op, expr } => Expression::Unary {
                op: *op,
                expr: Box::new(expr.borrow()),
            },
            OwnedExpression::Binary { op, lhs, rhs } => Expression::Binary {
                op: *op,
                lhs: Box::new(lhs.borrow()),
                rhs: Box::new(rhs.borrow()),
            },
            OwnedExpression::FunctionCall { ident, args } => Expression::FunctionCall {
                ident: ident.as_str(),
                args: args.iter().map(|arg| arg.borrow()).collect(),
            },
        }
    }
}

/// a value that can be assigned to a variable
#[derive(Debug, Clone)]
pub enum Value<'a> {
    Number(f64),
    String(String),
    Boolean(bool),
    Function {
        args: Vec<&'a str>,
        block: Block<'a>,
    },
    None,
}
#[derive(Debug, Clone)]
pub enum OwnedValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Function {
        args: Vec<String>,
        block: OwnedBlock,
    },
    None,
}
impl Value<'_> {
    pub fn to_owned(self) -> OwnedValue {
        match self {
            Value::Number(num) => OwnedValue::Number(num),
            Value::String(string) => OwnedValue::String(string),
            Value::Boolean(boolean) => OwnedValue::Boolean(boolean),
            Value::Function { args, block } => OwnedValue::Function {
                args: args.iter().map(|arg| arg.to_string()).collect(),
                block: block.to_owned(),
            },
            Value::None => OwnedValue::None,
        }
    }
}
impl OwnedValue {
    pub fn borrow(&self) -> Value {
        match self {
            OwnedValue::Number(num) => Value::Number(*num),
            OwnedValue::String(string) => Value::String(string.clone()),
            OwnedValue::Boolean(boolean) => Value::Boolean(*boolean),
            OwnedValue::Function { args, block } => Value::Function {
                args: args.iter().map(|arg| arg.as_str()).collect(),
                block: block.borrow(),
            },
            OwnedValue::None => Value::None,
        }
    }
}

impl core::fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Function { .. } => write!(f, "Function"),
            Value::Boolean(b) => {
                if *b {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Value::None => write!(f, "None"),
        }
    }
}

impl core::fmt::Display for OwnedValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.borrow().fmt(f)
    }
}
