use crate::fonts::{Cozette, Font};
use crate::types::*;
use crate::ui::*;
use alloc::vec::Vec;

pub struct Label<'a> {
    text: &'a str,
    hyphenate: bool,
}

impl Label<'_> {
    pub fn new(text: &str) -> Label {
        Label {
            text,
            hyphenate: true,
        }
    }

    pub fn hyphenate(mut self, hyphenate: bool) -> Self {
        self.hyphenate = hyphenate;
        self
    }
}

impl UIElement for Label<'_> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let max_width = context.placer.max_width();
        let chars_per_line = (max_width / Cozette::CHAR_WIDTH) as usize;
        if chars_per_line == 0 {
            return UIResult {
                rect: Rect::new(Position::new(0, 0), Position::new(0, 0)),
            };
        }

        let lines_hint = self.text.len() / chars_per_line.max(1);
        let max_visible_lines = context.fb.scaled_dimensions().height / Cozette::CHAR_HEIGHT;

        struct Line<'a> {
            text: &'a str,
            hyphenated: bool,
        }
        let mut lines = Vec::with_capacity(lines_hint.min(max_visible_lines as usize));

        for orig_line in self.text.lines() {
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
                        line_start += line_pos;
                        line_pos = 0;
                    }
                    while word_len > chars_per_line {
                        lines.push(Line {
                            text: &orig_line[line_start..line_start + chars_per_line],
                            hyphenated: self.hyphenate,
                        });
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
            }
        }

        let dimensions = Dimension {
            width: max_width,
            height: lines.len() as u32 * Cozette::CHAR_HEIGHT,
        };

        let rect = context.placer.alloc_space(dimensions);

        let mut position = rect.min;
        for line in lines {
            context
                .fb
                .draw_str::<Cozette>(&Color::new(255, 255, 255), line.text, &position);
            position.x += Cozette::CHAR_WIDTH as i64 * line.text.len() as i64;

            if line.hyphenated {
                context
                    .fb
                    .draw_char::<Cozette>(&Color::new(255, 255, 255), '-', &position);
            }

            position.x = rect.min.x;
            position.y += Cozette::CHAR_HEIGHT as i64;
        }

        UIResult { rect }
    }
}
