use monoscript::ast::{Statement, Block};
use monos_gfx::Image;
use nom_locate::LocatedSpan;
use hashbrown::HashMap;

pub type Span<'a> = LocatedSpan<&'a str>;


#[derive(Debug, Clone)]
pub struct MonoDoc<'a> {
    pub meta: MetaData<'a>,
    pub main_section: Section<'a>,
    pub sections: HashMap<&'a str, Section<'a>>,
}


#[derive(Debug, Clone)]
pub struct MetaData<'a> {
    pub name: Option<&'a str>,
    pub icon: Option<&'a str>,
    pub includes: Vec<&'a str>, 
}

#[derive(Debug, Clone)]
pub struct Section<'a> {
    pub position: Span<'a>,

    pub name: &'a str,
    pub content: Vec<Content<'a>>,
}

#[derive(Debug, Clone)]
pub struct Content<'a> {
    pub position: Span<'a>,

    pub kind: ContentKind<'a>,
}

#[derive(Debug, Clone)]
pub enum ContentKind<'a> {
    Heading(Vec<Inline<'a>>),
    Table(Vec<TableRow<'a>>),
    CodeBlock(&'a str),
    MonoscriptBlock(Block<'a>),
    Embed {
        target: EmbedTarget<'a>,
        anchor: EmbedAnchor,
    },
    Line(Vec<Inline<'a>>),
}

#[derive(Debug, Clone)]
pub struct TableRow<'a> {
    pub position: Span<'a>,

    pub cells: Vec<TableCell<'a>>,
}

#[derive(Debug, Clone)]
pub struct TableCell<'a> {
    pub position: Span<'a>,

    pub content: Vec<Inline<'a>>,
}

#[derive(Debug, Clone)]
pub struct Inline<'a> {
    pub position: Span<'a>,

    pub kind: InlineKind<'a>,
}

#[derive(Debug, Clone)]
pub enum InlineKind<'a> {
    Text(&'a str),
    Link {
        target: &'a str,
    },
    InlineEmbed {
        target: EmbedTarget<'a>,
    },
    Button {
        label: &'a str,
        action: Statement<'a>,
    },
}

#[derive(Debug, Clone)]
pub enum EmbedTarget<'a> {
    File(&'a str),
    Image(Image)
}

#[derive(Debug, Clone)]
pub enum EmbedAnchor {
    Top,
    Bottom,
    Left,
    Right,
    Fill,
}