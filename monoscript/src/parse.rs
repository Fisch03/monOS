use crate::ast::*;
use alloc::{boxed::Box, string::ToString, vec::Vec};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, anychar, char, line_ending, one_of, space0},
    combinator::{cut, map, map_res, opt, recognize, value},
    error::ParseError,
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, terminated, tuple},
    IResult, Parser,
};

fn identifier(input: Span) -> IResult<Span, Span> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))
    .parse(input)
}

fn eol_comment(input: Span) -> IResult<Span, ()> {
    value((), pair(tag("//"), is_not("\n\r"))).parse(input)
}

fn ws<'a, F, O, E: ParseError<Span<'a>>>(f: F) -> impl Parser<Span<'a>, O, E>
where
    F: Parser<Span<'a>, O, E>,
{
    delimited(space0, f, space0)
}

fn empty_line_end(input: Span) -> IResult<Span, ()> {
    value(
        (),
        tuple((space0, opt(eol_comment), many1(ws(line_ending)))),
    )(input)
}

impl<'a> Block<'a> {
    pub fn parse_scoped(input: Span<'a>) -> IResult<Span<'a>, Self> {
        delimited(
            pair(ws(char('{')), many0(empty_line_end)),
            Block::parse,
            ws(char('}')),
        )(input)
    }

    pub fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (rest, _) = many0(empty_line_end)(input)?;

        let (rest, statements) = ws(cut(separated_list0(
            many1(empty_line_end),
            Statement::parse,
        )))
        .parse(rest)?;

        let (rest, _) = many0(empty_line_end)(rest)?;

        Ok((
            rest,
            Block {
                span: input,
                statements,
            },
        ))
    }
}

impl<'a> Statement<'a> {
    pub fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        ws(alt((
            Self::parse_assignment,
            Self::parse_fn_assignment,
            Self::parse_return,
            Self::parse_hook,
            Self::parse_expression,
        )))
        .parse(input)
    }

    fn parse_assignment(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (rest, ident) = ws(identifier).parse(input)?;
        let (rest, kind) = ws(AssignmentKind::parse).parse(rest)?;
        let (rest, expr) = ws(cut(Expression::parse)).parse(rest)?;

        Ok((
            rest,
            Statement {
                span: input,
                kind: StatementKind::Assignment {
                    ident: ident.fragment(),
                    expression: expr,
                    kind,
                },
            },
        ))
    }

    fn parse_fn_assignment(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (rest, _) = tag("fn").parse(input)?;
        let (rest, ident) = cut(ws(identifier)).parse(rest)?;
        let (rest, args) = cut(delimited(
            char('('),
            separated_list0(char(','), ws(identifier)),
            char(')'),
        ))
        .parse(rest)?;

        let (rest, block) = Block::parse_scoped(rest)?;

        let args: Vec<&'a str> = args.into_iter().map(|s| s.into_fragment()).collect();

        Ok((
            rest,
            Statement {
                span: input,
                kind: StatementKind::Assignment {
                    ident: ident.fragment(),
                    expression: Expression::Literal(Value::Function { args, block }),
                    kind: AssignmentKind::Assign,
                },
            },
        ))
    }

    fn parse_return(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (rest, _) = tag("return").parse(input)?;
        let (rest, expr) = opt(ws(Expression::parse)).parse(rest)?;

        Ok((
            rest,
            Statement {
                span: input,
                kind: StatementKind::Return { expression: expr },
            },
        ))
    }

    fn parse_hook(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (rest, kind) = identifier(input)?;
        let (rest, params) = delimited(
            char('('),
            separated_list0(char(','), ws(Expression::parse)),
            char(')'),
        )(rest)
        .unwrap_or((rest, Vec::new()));

        let (rest, block) = Block::parse_scoped(rest)?;

        Ok((
            rest,
            Statement {
                span: input,
                kind: StatementKind::Hook {
                    kind: &kind,
                    params,
                    block,
                },
            },
        ))
    }

    fn parse_expression(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (rest, expr) = ws(Expression::parse).parse(input)?;
        Ok((
            rest,
            Statement {
                span: input,
                kind: StatementKind::Expression(expr),
            },
        ))
    }
}

impl<'a> Expression<'a> {
    pub fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (rest, expr) = alt((
            Self::parse_function_call,
            Value::parse_literal.map(Expression::Literal),
            identifier.map(|s| Expression::Identifier(s.fragment())),
            Self::parse_unary,
        ))(input)?;

        let (rest, mut expr) = Self::parse_binary(rest, &expr).unwrap_or((rest, expr));

        expr.fix_order();

        Ok((rest, expr))
    }

    fn fix_order(&mut self) {
        match self {
            Expression::Binary { op, lhs, rhs } => {
                if let Expression::Binary {
                    op: rhs_op,
                    lhs: rhs_lhs,
                    rhs: rhs_rhs,
                } = &mut **rhs
                {
                    if op.precedence() <= rhs_op.precedence() {
                        let new_rhs = Expression::Binary {
                            op: *op,
                            lhs: lhs.clone(),
                            rhs: rhs_lhs.clone(),
                        };
                        **lhs = new_rhs;
                        **rhs = *rhs_rhs.clone();
                    }
                }
            }
            _ => {}
        }
    }

    fn parse_args(input: Span<'a>) -> IResult<Span<'a>, Vec<Expression<'a>>> {
        delimited(
            char('('),
            separated_list0(char(','), ws(Expression::parse)),
            char(')'),
        )(input)
    }

    fn parse_unary(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (rest, op) = ws(UnaryOp::parse).parse(input)?;
        let (rest, expr) = ws(Expression::parse).parse(rest)?;
        Ok((
            rest,
            Expression::Unary {
                op,
                expr: Box::new(expr),
            },
        ))
    }

    fn parse_binary(input: Span<'a>, maybe_lhs: &Expression<'a>) -> IResult<Span<'a>, Self> {
        let (rest, op) = ws(BinaryOp::parse).parse(input)?;
        let (rest, rhs) = ws(Expression::parse).parse(rest)?;
        Ok((
            rest,
            Expression::Binary {
                op,
                lhs: Box::new(maybe_lhs.clone()),
                rhs: Box::new(rhs),
            },
        ))
    }

    fn parse_function_call(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (rest, ident) = identifier(input)?;
        let (rest, args) = Expression::parse_args.parse(rest)?;

        Ok((
            rest,
            Expression::FunctionCall {
                ident: ident.fragment(),
                args,
            },
        ))
    }
}

impl<'a> Value<'a> {
    pub fn parse_literal(input: Span<'a>) -> IResult<Span<'a>, Self> {
        alt((Self::parse_number, Self::parse_string, Self::parse_boolean))(input)
    }

    fn parse_number(input: Span<'a>) -> IResult<Span<'a>, Self> {
        map_res(
            recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
            |s: Span| s.fragment().parse::<f64>().map(Value::Number),
        )
        .parse(input)
    }

    fn parse_string(input: Span<'a>) -> IResult<Span<'a>, Self> {
        delimited(
            char('"'),
            map(recognize(is_not("\"")), |s: Span| {
                Value::String(s.fragment().to_string())
            }),
            char('"'),
        )(input)
    }

    fn parse_boolean(input: Span<'a>) -> IResult<Span<'a>, Self> {
        alt((
            tag("true").map(|_| Value::Boolean(true)),
            tag("false").map(|_| Value::Boolean(false)),
        ))(input)
    }
}

impl AssignmentKind {
    pub fn parse<'a>(input: Span<'a>) -> IResult<Span<'a>, Self> {
        alt((
            tag("=").map(|_| AssignmentKind::Assign),
            tag("+=").map(|_| AssignmentKind::AddAssign),
            tag("-=").map(|_| AssignmentKind::SubAssign),
            tag("*=").map(|_| AssignmentKind::MulAssign),
            tag("/=").map(|_| AssignmentKind::DivAssign),
        ))(input)
    }
}

impl UnaryOp {
    pub fn parse<'a>(input: Span<'a>) -> IResult<Span<'a>, Self> {
        alt((
            tag("!").map(|_| UnaryOp::Not),
            tag("-").map(|_| UnaryOp::Neg),
        ))(input)
    }
}

impl BinaryOp {
    pub fn parse<'a>(input: Span<'a>) -> IResult<Span<'a>, Self> {
        alt((
            tag("+").map(|_| BinaryOp::Add),
            tag("-").map(|_| BinaryOp::Sub),
            tag("*").map(|_| BinaryOp::Mul),
            tag("/").map(|_| BinaryOp::Div),
            tag("%").map(|_| BinaryOp::Mod),
            tag("==").map(|_| BinaryOp::Eq),
            tag("!=").map(|_| BinaryOp::Ne),
            tag("<").map(|_| BinaryOp::Lt),
            tag("<=").map(|_| BinaryOp::Le),
            tag(">").map(|_| BinaryOp::Gt),
            tag(">=").map(|_| BinaryOp::Ge),
            tag("&&").map(|_| BinaryOp::And),
            tag("||").map(|_| BinaryOp::Or),
        ))(input)
    }
}
