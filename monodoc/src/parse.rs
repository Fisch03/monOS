use crate::ast::*;
use nom::{
    branch::alt, bytes::complete::{tag, take_until, take_while}, character::is_alphanumeric, combinator::opt, multi::{many0, many1}, sequence::{delimited, terminated}, IResult
};
use nom_locate::position;


impl<'a> MonoDoc<'a> {
    pub fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (input, meta) = MetaData::parse(input)?;
        let (input, main_section) = Section::parse_main(input)?;
        let (input, sections) = many0(Section::parse)(input)?;
       
        Ok((input, MonoDoc { meta, main_section, sections }))
    }
}

impl<'a> MetaData<'a> {
    pub fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        // TODO
        Ok((input, MetaData {
            name: None,
            icon: None,
            includes: Vec::new(),
        }))
    }
}

impl<'a> Section<'a> {
    fn parse_heading(input: Span<'a>) -> IResult<Span<'a>, &'a str> {
       delimited(tag("=== "), take_while(is_alphanumeric), tag(" ==="))(input)
    }

    fn parse_content(input: Span<'a>) -> IResult<Span<'a>, Vec<Content<'a>>> {
        many0(Content::parse)(input)
    }

    pub fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (input, name) = Self::parse_heading(input)?;
        let (input, position) = position(input)?;
        let (input, content) = Self::parse_content(input)?;
        
        Ok((input, Section { name, content, position }))
    }

    pub fn parse_main(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let input = match opt(Self::parse_heading(input)) {
            Ok((input, Some("main"))) => input,
            Ok((input, None)) => input,
            _ => return Err(nom::Err::Error(input)),
        };

        let (input, position) = position(input)?;
        let (input, content) = Self::parse_content(input)?;

        Ok((input, Section { name: "main", content, position }))
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

    fn parse_heading(input: Span<'a>) -> IResult<Span<'a>, ContentKind> {
        let (input, content) = delimited(
            tag("# "),
            many0(Inline::parse),
            tag("\n"),
        )(input)?;
        Ok((input, ContentKind::Heading(content)))
    }

    fn parse_table(input: Span<'a>) -> IResult<Span<'a>, ContentKind> {
        let (input, content) = many1(terminated(
            TableRow::parse,
            tag("\n"),
        ))(input)?;
        Ok((input, ContentKind::Table(content)))
    }

    fn parse_monoscript_block(input: Span<'a>) -> IResult<Span<'a>, ContentKind> {
        let (input, content) = delimited(
            tag("```"),
            take_until("```"),
            tag("```"),
        )(input)?;

        let (rest,content) = monoscript::ast::Block::parse(content.fragment())?;
        if !rest.is_empty() {
            return Err(nom::Err::Error(input));
        }

        Ok((input, ContentKind::MonoscriptBlock(content.fragment())))
    }

    fn parse_code_block(input: Span<'a>) -> IResult<Span<'a>, ContentKind> {
        let (input, content) = delimited(
            tag("```"),
            take_until("```"),
            tag("```"),
        )(input)?;
        Ok((input, ContentKind::CodeBlock(content.fragment())))
    }

    fn parse_embed(input: Span<'a>) -> IResult<Span<'a>, ContentKind> {
        let (input, content) = delimited(
            tag("![["),
            take_until("]]"),
            tag("]]\n"),
        )(input)?;

        let (target, anchor) = match content.fragment().split_once(',') {
            Some((target, anchor)) => (target, EmbedAnchor::parse(anchor)),
            None => (content.fragment(), EmbedAnchor::Fill),
        };
        
        Ok((input, ContentKind::Embed { target, anchor }))
    }

    fn parse_line(input: Span<'a>) -> IResult<Span<'a>, ContentKind> {
        let (input, content) = terminated(
            many0(Inline::parse),
            tag("\n"),
        )(input)?;
        Ok((input, ContentKind::Line(content)))
    }
}

impl<'a> TableRow<'a> {
    fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (input, position) = position(input)?;
        let (input, cells) = preceded(
            tag("|"),
            many1(TableCell::parse),
        )(input)?;
        Ok((input, TableRow { cells, position }))
    }
}

impl<'a> TableCell<'a> {
    fn parse(input: Span<'a>) -> IResult<Span<'a>, Self> {
        let (input, position) = position(input)?;
        let (input, content) = terminated(
            many0(Inline::parse),
            tag("|"),
        )(input)?;
        
        Ok((input, TableCell { content, position }))
    }
}

impl EmbedAnchor {
    fn parse(input: &str) -> Option<Self> {
        match input {
            "top" => Some(EmbedAnchor::Top),
            "bottom" => Some(EmbedAnchor::Bottom),
            "left" => Some(EmbedAnchor::Left),
            "right" => Some(EmbedAnchor::Right),
            "fill" => Some(EmbedAnchor::Fill),
            _ => None,
        }
    }
}