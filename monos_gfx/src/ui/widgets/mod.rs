mod label;
pub use label::Label;

mod button;
pub use button::{Button, ImageButton};

use crate::fonts::{Cozette, Font};
use crate::types::*;
use crate::Framebuffer;

pub enum TextWrap {
    Disabled,
    Enabled { hyphenate: bool },
}

pub struct Line<'a> {
    pub text: &'a str,
    pub hyphenated: bool,
}
pub struct Lines<'a> {
    lines: Vec<Line<'a>>,
    dimensions: Dimension,
}
impl<'a> Lines<'a> {
    #[inline]
    pub fn dimensions(&self) -> Dimension {
        self.dimensions
    }

    pub fn iter(&self) -> impl Iterator<Item = &Line<'a>> {
        self.lines.iter()
    }

    pub fn layout_single_line(text: &'a str, max_width: u32) -> Self {
        if max_width < Cozette::CHAR_WIDTH {
            return Self {
                lines: Vec::new(),
                dimensions: Dimension::new(0, 0),
            };
        }
        let chars_per_line = (max_width / Cozette::CHAR_WIDTH) as usize;

        let text = &text[..chars_per_line.min(text.len())];

        Self {
            lines: vec![Line {
                text,
                hyphenated: false,
            }],
            dimensions: Dimension {
                width: Cozette::CHAR_WIDTH as u32 * text.len() as u32,
                height: Cozette::CHAR_HEIGHT as u32,
            },
        }
    }

    pub fn layout_wrapped(text: &'a str, hyphenate: bool, max_dimensions: Dimension) -> Self {
        let chars_per_line = (max_dimensions.width / Cozette::CHAR_WIDTH) as usize;
        if chars_per_line == 0 {
            return Self {
                lines: Vec::new(),
                dimensions: Dimension::new(0, 0),
            };
        }

        let lines_hint = text.len() / chars_per_line.max(1);
        let max_visible_lines = max_dimensions.height / Cozette::CHAR_HEIGHT;

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
            longest_line as u32 * Cozette::CHAR_WIDTH,
            lines.len() as u32 * Cozette::CHAR_HEIGHT,
        );
        Self { lines, dimensions }
    }

    pub fn draw(&self, fb: &mut Framebuffer, position: Position, color: Color) {
        let mut position = position;
        for line in self.iter() {
            fb.draw_str::<Cozette>(&color, line.text, &position);
            position.x += Cozette::CHAR_WIDTH as i64 * line.text.len() as i64;
            if line.hyphenated {
                fb.draw_char::<Cozette>(&color, '-', &position);
            }
            position.x = position.x.min(fb.dimensions().width as i64);
            position.y += Cozette::CHAR_HEIGHT as i64;
        }
    }
}
