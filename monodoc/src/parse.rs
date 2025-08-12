use crate::ast::*;
use alloc::vec::Vec;
use hashbrown::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    error::{context, ErrorKind},
    multi::{many0, many1},
    sequence::{delimited, preceded, separated_pair, terminated},
    AsChar, Err, IResult, Parser,
};
use nom_locate::position;

impl<'a> MonoDoc<'a> {
    pub fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (input, meta) = MetaData::parse(input)?;
        let (input, main_section) = Section::parse_main(input)?;
        let (input, sections) = many0(Section::parse)(input)?;

        let sections = sections
            .into_iter()
            .map(|section| (section.name, section))
            .collect::<HashMap<_, _>>();

        Ok((
            input,
            MonoDoc {
                meta,
                main_section,
                sections,
            },
        ))
    }
}

impl<'a> MetaData<'a> {
    pub fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        // TODO
        Ok((
            input,
            MetaData {
                name: None,
                icon: None,
                includes: Vec::new(),
            },
        ))
    }
}

impl<'a> Section<'a> {
    fn parse_heading(input: Span<'a>) -> IResult<Span<'a>, &'a str> {
        delimited(tag("=== "), take_while(AsChar::is_alphanum), tag(" ==="))(input).map(
            |(input, name)| {
                let name = name.fragment();
                let name = if name.is_empty() { "main" } else { name };
                (input, name)
            },
        )
    }

    fn parse_content(input: Span<'a>) -> IResult<Span<'a>, Vec<Content<'a>>> {
        many0(Content::parse)(input)
    }

    pub fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (input, name) = Self::parse_heading(input)?;
        let (input, position) = position(input)?;
        let (input, content) = Self::parse_content(input)?;

        Ok((
            input,
            Section {
                name,
                content,
                position,
            },
        ))
    }

    pub fn parse_main(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (input, name) = Self::parse_heading(input)?;
        if name != "main" {
            return Err(Err::Error(ErrorKind::Tag));
        }

        let (input, position) = position(input)?;
        let (input, content) = Self::parse_content(input)?;

        Ok((
            input,
            Section {
                name: "main",
                content,
                position,
            },
        ))
    }
}

impl<'a> Content<'a> {
    fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (input, position) = position(input)?;
        let (input, kind) = alt((
            // # ...
            Self::parse_heading,
            // |...|
            Self::parse_table,
            // ```!...```
            Self::parse_monoscript_block,
            // ```...```
            Self::parse_code_block,
            // ![[...]]
            Self::parse_embed,
            Self::parse_line,
        ))(input)?;

        Ok((input, Content { position, kind }))
    }

    fn parse_heading(input: Span<'a>) -> IResult<Span<'a>, ContentKind<'a>> {
        let (input, content) = delimited(tag("# "), many0(Inline::parse), tag("\n"))(input)?;
        Ok((input, ContentKind::Heading(content)))
    }

    fn parse_table(input: Span<'a>) -> IResult<Span<'a>, ContentKind<'a>> {
        let (input, content) = many1(terminated(TableRow::parse, tag("\n")))(input)?;
        Ok((input, ContentKind::Table(content)))
    }

    fn parse_monoscript_block(input: Span<'a>) -> IResult<Span<'a>, ContentKind<'a>> {
        let (input, content) = delimited(tag("```"), take_until("```"), tag("```"))(input)?;

        let (_, content) = monoscript::ast::Block::parse(content)?;

        Ok((input, ContentKind::MonoscriptBlock(content)))
    }

    fn parse_code_block(input: Span<'a>) -> IResult<Span<'a>, ContentKind<'a>> {
        let (input, content) = delimited(tag("```"), take_until("```"), tag("```"))(input)?;
        Ok((input, ContentKind::CodeBlock(content.fragment())))
    }

    fn parse_embed(input: Span<'a>) -> IResult<Span<'a>, ContentKind<'a>> {
        let (input, (target, anchor)) = delimited(
            tag("![["),
            separated_pair(EmbedTarget::parse, tag(","), EmbedAnchor::parse),
            tag("]]\n"),
        )(input)?;

        Ok((input, ContentKind::Embed { target, anchor }))
    }

    fn parse_line(input: Span<'a>) -> IResult<Span<'a>, ContentKind<'a>> {
        let (input, content) = terminated(many0(Inline::parse), tag("\n"))(input)?;
        Ok((input, ContentKind::Line(content)))
    }
}

impl<'a> TableRow<'a> {
    fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (input, position) = position(input)?;
        let (input, cells) = preceded(tag("|"), many1(TableCell::parse))(input)?;
        Ok((input, TableRow { cells, position }))
    }
}

impl<'a> TableCell<'a> {
    fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (input, position) = position(input)?;
        let (input, content) = terminated(many0(Inline::parse), tag("|"))(input)?;

        Ok((input, TableCell { content, position }))
    }
}

impl EmbedAnchor {
    fn parse<'a>(input: Span<'a>) -> IResult<Span<'a>, Self> {
        alt((
            tag("fill").map(|_| EmbedAnchor::Fill),
            tag("left").map(|_| EmbedAnchor::Left),
            tag("right").map(|_| EmbedAnchor::Right),
            tag("top").map(|_| EmbedAnchor::Top),
            tag("bottom").map(|_| EmbedAnchor::Bottom),
        ))(input)
    }
}
