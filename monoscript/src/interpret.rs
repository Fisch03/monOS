use alloc::{string::String, vec::Vec};

use crate::ast::{ASTIter, ASTNode, ASTNodeKind, ScriptAST};

enum Keywords {
    If,
    While,
    Window,
    Function,
}
impl Keywords {
    fn is_tl_keyword(ident: &str) -> Option<Self> {
        match ident {
            "window" => Some(Self::Window),
            _ => None,
        }
        .or_else(|| Self::is_keyword(ident))
    }

    fn is_keyword(ident: &str) -> Option<Self> {
        match ident {
            "if" => Some(Self::If),
            "while" => Some(Self::While),
            "window" => Some(Self::Window),
            "function" | "fn" => Some(Self::Function),
            _ => None,
        }
    }

    fn parse<'a>(
        self,
        ast: &mut ASTIter<'a, '_>,
        script: &mut Script<'a>,
    ) -> Result<Option<Statement<'a>>, InterpretError<'a>> {
        let res = match self {
            Self::If => Some(Statement::parse_if(ast, script)?),
            Self::Window => match ast.next() {
                Some(node) => match &node.kind {
                    ASTNodeKind::Block(block) => {
                        let window = Window {
                            render: CodeBlock::parse(ASTIter::new(block), script, false)?,
                        };
                        script.window = Some(window);
                        None
                    }
                    _ => {
                        return Err(InterpretError::ParseError(
                            node.clone(),
                            "expected window block",
                        ))
                    }
                },
                None => return Err(InterpretError::UnexpectedEOF("expected window block")),
            },
            _ => todo!(),
        };

        Ok(res)
    }
}

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Number(usize),
    String(String),
    Boolean(bool),
    Function {
        args: Vec<&'a str>,
        body: CodeBlock<'a>,
    },
    None, // this isn't a null value, but rather the return value of a function that returns nothing
}
impl Value<'_> {
    pub fn is_constant(node: &ASTNode) -> Option<Self> {
        match node.kind {
            ASTNodeKind::Ident(ident) => match ident {
                "true" => Some(Self::Boolean(true)),
                "false" => Some(Self::Boolean(false)),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeBlock<'a> {
    statements: Vec<Statement<'a>>,
}
impl<'a> CodeBlock<'a> {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    pub fn insert(&mut self, statement: Statement<'a>) {
        self.statements.push(statement);
    }

    pub fn parse(
        mut ast: ASTIter<'a, '_>,
        script: &mut Script<'a>,
        is_tl: bool,
    ) -> Result<Self, InterpretError<'a>> {
        let mut parsed_block = Self::new();
        let is_keyword_fn = if is_tl {
            Keywords::is_tl_keyword
        } else {
            Keywords::is_keyword
        };

        loop {
            let node = match ast.next() {
                Some(node) => node,
                None => break,
            };
            match node.kind {
                ASTNodeKind::Ident(ident) => match is_keyword_fn(ident) {
                    Some(keyword) => if let Some(statement) = keyword.parse(&mut ast, script)? {
                        parsed_block.insert(statement);
                    },
                    None => {
                        match ast.next() {
                            Some(node) => match &node.kind {
                                ASTNodeKind::Ident("=") => {
                                    let expr = Expression::parse(&mut ast)?;
                                    parsed_block.insert(Statement::Assignment(ident, expr));
                                }
                                ASTNodeKind::Block(block) => {
                                    let mut args = Vec::new();
                                    let mut block_ast = ASTIter::new(block);
                                    while let Some(node) = block_ast.peek() {
                                        match node.kind {
                                            ASTNodeKind::Ident(",") => {block_ast.next();},
                                            _ => {
                                                let arg = Expression::parse(&mut block_ast)?;
                                                args.push(arg);
                                            }
                                        }
                                    }
                                    parsed_block.insert(Statement::FunctionCall(ident, args));
                                }
                                _ => return Err(InterpretError::ParseError(node.clone(), "expected assignment")),
                            },
                            None => return Err(InterpretError::UnexpectedEOF("expected assignment or function call")),
                        }
                    },
                },
                ASTNodeKind::ConfigSeparator => return Err(InterpretError::ParseError(node.clone(), "the configuration separator '---' is a reserved keyword. please remove it from your script")),
                _ => return Err(InterpretError::ParseError(node.clone(), "unexpected token")),
            }
        }

        Ok(parsed_block)
    }
}
impl<'a> core::ops::Deref for CodeBlock<'a> {
    type Target = Vec<Statement<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.statements
    }
}

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Assignment(&'a str, Expression<'a>),
    Expression(Expression<'a>),
    If(Expression<'a>, CodeBlock<'a>, Option<CodeBlock<'a>>),
    While(Expression<'a>, CodeBlock<'a>),
    FunctionCall(&'a str, Vec<Expression<'a>>),
}
impl<'a> Statement<'a> {
    pub fn parse_if(
        ast: &mut ASTIter<'a, '_>,
        script: &mut Script<'a>,
    ) -> Result<Self, InterpretError<'a>> {
        let condition = match ast.next() {
            Some(node) => match &node.kind {
                ASTNodeKind::Block(block) => Expression::parse(&mut ASTIter::new(block))?,
                _ => {
                    return Err(InterpretError::ParseError(
                        node.clone(),
                        "expected if condition block",
                    ))
                }
            },
            None => return Err(InterpretError::UnexpectedEOF("expected if condition block")),
        };

        let if_block = match ast.next() {
            Some(node) => match &node.kind {
                ASTNodeKind::Block(block) => CodeBlock::parse(ASTIter::new(block), script, false)?,
                _ => {
                    return Err(InterpretError::ParseError(
                        node.clone(),
                        "expected if block",
                    ))
                }
            },
            None => return Err(InterpretError::UnexpectedEOF("expected if block")),
        };

        let else_block = match ast.peek() {
            Some(node) => match node.kind {
                ASTNodeKind::Ident("else") => {
                    ast.next();
                    match ast.next() {
                        Some(node) => match &node.kind {
                            ASTNodeKind::Block(block) => {
                                Some(CodeBlock::parse(ASTIter::new(block), script, false)?)
                            }
                            _ => {
                                return Err(InterpretError::ParseError(
                                    node.clone(),
                                    "expected else block",
                                ))
                            }
                        },
                        None => return Err(InterpretError::UnexpectedEOF("expected else block")),
                    }
                }
                _ => None,
            },
            None => None,
        };

        Ok(Self::If(condition, if_block, else_block))
    }
}

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Literal(Value<'a>),
    Ident(&'a str),
    //BinaryOp(BinaryOp, Box<Expression<'a>>, Box<Expression<'a>>),
    //UnaryOp(UnaryOp, Box<Expression<'a>>),
}
impl<'a> Expression<'a> {
    pub fn parse(ast: &mut ASTIter<'a, '_>) -> Result<Self, InterpretError<'a>> {
        match ast.next() {
            Some(node) => match &node.kind {
                ASTNodeKind::Literal(value) => Ok(Self::Literal(value.clone())),
                ASTNodeKind::Ident(ident) => {
                    if let Some(value) = Value::is_constant(node) {
                        return Ok(Self::Literal(value));
                    }
                    Ok(Self::Ident(ident))
                }
                _ => Err(InterpretError::ParseError(
                    node.clone(),
                    "expected expression",
                )),
            },
            None => Err(InterpretError::UnexpectedEOF("expected expression")),
        }
    }
}

#[derive(Debug)]
pub struct Window<'a> {
    pub(crate) render: CodeBlock<'a>,
}

#[derive(Debug)]
pub struct Script<'a> {
    pub code: CodeBlock<'a>,
    pub window: Option<Window<'a>>,
}

#[derive(Debug)]
pub enum InterpretError<'a> {
    ParseError(ASTNode<'a>, &'static str),
    UnexpectedEOF(&'static str),
    UndefinedVariable(&'a str),
    InvalidOperation,
}
impl<'a> Script<'a> {
    pub fn empty() -> Self {
        Self {
            code: CodeBlock::new(),
            window: None,
        }
    }

    pub fn interpret(ast: ScriptAST<'a>) -> Result<Self, InterpretError<'a>> {
        let ast = ASTIter::new(&ast.nodes);
        let mut script = Self::empty();

        script.code = CodeBlock::parse(ast, &mut script, false)?;

        Ok(script)
    }
}
