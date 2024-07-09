mod label;
pub use label::Label;

mod textbox;
pub use textbox::Textbox;

mod button;
pub use button::{Button, ImageButton};

use crate::types::*;
use crate::Font;
use crate::Framebuffer;
use core::marker::PhantomData;

pub enum TextWrap {
    Disabled,
    Enabled { hyphenate: bool },
    Everywhere,
}

pub struct Line<'a> {
    pub text: &'a str,
    pub hyphenated: bool,
}

pub struct Lines<'a, F>
where
    F: Font,
{
    lines: Vec<Line<'a>>,
    dimensions: Dimension,
    font: PhantomData<F>,
}
impl<'a, F: Font> Lines<'a, F> {
    #[inline]
    pub fn dimensions(&self) -> Dimension {
        self.dimensions
    }

    pub fn iter(&self) -> impl Iterator<Item = &Line<'a>> {
        self.lines.iter()
    }

    pub fn layout(text: &'a str, wrap: TextWrap, max_dimensions: Dimension) -> Self {
        match wrap {
            TextWrap::Disabled => Lines::<F>::layout_single_line(text, max_dimensions.width),
            TextWrap::Enabled { hyphenate } => {
                Lines::<F>::layout_wrapped(text, hyphenate, max_dimensions)
            }
            TextWrap::Everywhere => Lines::<F>::layout_wrapped_everywhere(text, max_dimensions),
        }
    }

    pub fn layout_single_line(text: &'a str, max_width: u32) -> Self {
        if max_width < F::CHAR_WIDTH {
            return Self {
                lines: Vec::new(),
                dimensions: Dimension::new(0, 0),
                font: PhantomData,
            };
        }
        let chars_per_line = (max_width / F::CHAR_WIDTH) as usize;

        let text = &text[..chars_per_line.min(text.len())];

        Self {
            lines: vec![Line {
                text,
                hyphenated: false,
            }],
            dimensions: Dimension {
                width: F::CHAR_WIDTH as u32 * text.len() as u32,
                height: F::CHAR_HEIGHT as u32,
            },
            font: PhantomData,
        }
    }

    pub fn layout_wrapped_everywhere(text: &'a str, max_dimensions: Dimension) -> Self {
        let chars_per_line = (max_dimensions.width / F::CHAR_WIDTH) as usize;

        text.lines().fold(
            Self {
                lines: Vec::new(),
                dimensions: Dimension::new(0, F::CHAR_HEIGHT),
                font: PhantomData,
            },
            |mut lines, mut orig_line| {
                while orig_line.len() > chars_per_line {
                    lines.lines.push(Line {
                        text: &orig_line[..chars_per_line],
                        hyphenated: false,
                    });

                    lines.dimensions.width = chars_per_line as u32 * F::CHAR_WIDTH;

                    lines.dimensions.height += F::CHAR_HEIGHT;
                    orig_line = &orig_line[chars_per_line..];
                }

                if orig_line.is_empty() {
                    return lines;
                }

                let remaining_width = orig_line.len() as u32 * F::CHAR_WIDTH;
                lines.lines.push(Line {
                    text: orig_line,
                    hyphenated: false,
                });
                lines.dimensions.width = lines.dimensions.width.max(remaining_width);

                lines
            },
        )
    }

    pub fn layout_wrapped(text: &'a str, hyphenate: bool, max_dimensions: Dimension) -> Self {
        let chars_per_line = (max_dimensions.width / F::CHAR_WIDTH) as usize;
        if chars_per_line == 0 {
            return Self {
                lines: Vec::new(),
                dimensions: Dimension::new(0, 0),
                font: PhantomData,
            };
        }

        let lines_hint = text.len() / chars_per_line.max(1);
        let max_visible_lines = max_dimensions.height / F::CHAR_HEIGHT;

        let mut lines = Vec::with_capacity(lines_hint.min(max_visible_lines as usize));
        let mut longest_line = 0;

        for orig_line in text.lines() {
            let mut line_start = 0;
            let mut line_pos = 0;

            for word in orig_line.split(&[' ', '-']) {
                let mut word_len = word.len();
                if word_len > chars_per_line {
                    if line_pos > 0 {
                        lines.push(Line {
                            text: &orig_line[line_start..line_start + line_pos],
                            hyphenated: false,
                        });
                        longest_line = longest_line.max(line_pos);
                        line_start += line_pos;
                        line_pos = 0;
                    }
                    while word_len > chars_per_line {
                        lines.push(Line {
                            text: &orig_line[line_start..line_start + chars_per_line],
                            hyphenated: hyphenate,
                        });
                        longest_line = longest_line.max(chars_per_line);
                        word_len -= chars_per_line;
                        line_start += chars_per_line;
                        line_pos = 0;
                    }
                }

                if line_pos + word_len > chars_per_line {
                    lines.push(Line {
                        text: &orig_line[line_start..line_start + line_pos],
                        hyphenated: false,
                    });
                    longest_line = longest_line.max(line_pos);
                    line_start += line_pos;
                    line_pos = 0;
                }

                line_pos += word_len + 1;
            }

            if lines.len() >= max_visible_lines as usize {
                break;
            }

            if line_start < orig_line.len() {
                lines.push(Line {
                    text: &orig_line[line_start..],
                    hyphenated: false,
                });
                longest_line = longest_line.max(line_pos);
            }
        }

        let dimensions = Dimension::new(
            longest_line as u32 * F::CHAR_WIDTH,
            lines.len() as u32 * F::CHAR_HEIGHT,
        );
        Self {
            lines,
            dimensions,
            font: PhantomData,
        }
    }

    pub fn char_position(&self, index: usize) -> Position {
        let mut curr_index = 0;

        for (line_index, line) in self.lines.iter().enumerate() {
            if curr_index + line.text.len() >= index {
                let char_index = index - curr_index;
                return Position {
                    x: char_index as i64 * F::CHAR_WIDTH as i64,
                    y: line_index as i64 * F::CHAR_HEIGHT as i64,
                };
            }
            curr_index += line.text.len();
        }

        Position {
            x: self.lines.last().map_or(0, |line| line.text.len()) as i64 * F::CHAR_WIDTH as i64,
            y: self.lines.len() as i64 * F::CHAR_HEIGHT as i64,
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, position: Position, color: Color) {
        let mut curr_position = position;
        for line in self.iter() {
            fb.draw_str::<F>(&color, line.text, &curr_position);
            curr_position.x += F::CHAR_WIDTH as i64 * line.text.len() as i64;
            if line.hyphenated {
                fb.draw_char::<F>(&color, '-', &curr_position);
            }
            curr_position.x = position.x;
            curr_position.y += F::CHAR_HEIGHT as i64;
        }
    }
}
