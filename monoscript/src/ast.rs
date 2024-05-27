use nom_locate::LocatedSpan;

use alloc::{boxed::Box, string::String, vec::Vec};

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

/// ways to assign a value to a variable
#[derive(Debug, Clone, Copy)]
pub enum AssignmentKind {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

/// hooks are special code blocks that get called by the runtime when certain events occur.
/// they are the main way to interact with the outside world
#[derive(Debug, Clone)]
pub enum HookType {
    Window,
    Key(char),
}

pub type Span<'a> = LocatedSpan<&'a str>;

/// a list of statements (think: function body, etc)
#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub span: Span<'a>,
    pub statements: Vec<Statement<'a>>,
}

/// a single statement (think: a line of code)
#[derive(Debug, Clone)]
pub struct Statement<'a> {
    pub span: Span<'a>,
    pub kind: StatementKind<'a>,
}

#[derive(Debug, Clone)]
pub enum StatementKind<'a> {
    Assignment {
        ident: &'a str,
        expression: Expression<'a>,
        kind: AssignmentKind,
    },
    FunctionCall {
        ident: &'a str,
        args: Vec<Expression<'a>>,
    },
    Hook {
        kind: HookType,
        block: Block<'a>,
    },
    If {
        condition: Expression<'a>,
        block: Block<'a>,
        else_block: Option<Block<'a>>,
    },
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
