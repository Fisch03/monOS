use super::*;
use monos_gfx::{
    font,
    input::KeyCode,
    ui::{Direction, MarginMode, UIFrame},
    Color, Framebuffer, Image, Input, Position, Rect,
};

use core::sync::atomic::Ordering;

const SCREEN_RECT: Rect = Rect::new(Position::new(0, 0), Position::new(640, 480));
const RENDER_TIMEOUT: u64 = 16;

#[derive(Debug)]
pub struct Window {
    // icon: Image,
    pos: Position,
    chunk: MemoryMappedChunk<WindowChunk>,
    target_handle: ChannelHandle,
}

impl Window {
    fn rect(&self) -> Rect {
        Rect::new(self.pos, self.pos + self.chunk.dimensions)
    }

    fn header_rect(&self) -> Rect {
        Rect::new(
            Position::new(self.pos.x, self.pos.y - 16),
            Position::new(self.pos.x + self.chunk.dimensions.width as i64, self.pos.y),
        )
    }

    fn full_rect(&self) -> Rect {
        Rect::new(
            Position::new(self.pos.x, self.pos.y - 16),
            Position::new(
                self.pos.x + self.chunk.dimensions.width as i64,
                self.pos.y + self.chunk.dimensions.height as i64,
            ),
        )
        .grow(1)
    }
}

#[derive(Debug)]
struct ScreenArea {
    rect: Rect,
    window: usize,
}

#[must_use]
pub struct RenderResult {
    pub hide_cursor: bool,
}

pub struct WindowServer {
    windows: Vec<Window>,
    close_button: Image,

    recv_handle: PartialReceiveChannelHandle,

    drag_start: Option<Position>,

    screen_areas: Vec<ScreenArea>,
    areas_changed: bool,

    next_window_id: u64,
    last_render: u64,

    mouse_grabbed: bool,

    debug: bool,
}

impl WindowServer {
    pub fn new(port: &str) -> Self {
        let recv_handle = syscall::serve(port).unwrap();

        let close_button = File::open("data/close.ppm").unwrap();
        let close_button = Image::from_ppm(&close_button).unwrap();

        WindowServer {
            windows: Vec::new(),
            close_button,
            next_window_id: 0,
            recv_handle,
            drag_start: None,
            screen_areas: Vec::new(),
            areas_changed: false,
            last_render: 0,
            debug: false,
            mouse_grabbed: false,
        }
    }

    // safety: msg must be a WindowServerMessage
    pub unsafe fn handle_message(&mut self, msg: GenericMessage) {
        let sender = msg.sender;
        let msg = unsafe { WindowClientMessage::from_message(msg) };
        let msg = match msg {
            Some(msg) => msg,
            None => return,
        };

        match msg {
            WindowClientMessage::CreateWindow {
                dimensions,
                creation_id,
            } => {
                let id = self.next_window_id;
                self.next_window_id += 1;

                let rect = Rect::centered_in(SCREEN_RECT, dimensions);

                let chunk = syscall::request_chunk::<WindowChunk>().unwrap();
                let mut chunk = chunk.make_mmapped();
                chunk.id = id;
                chunk.dimensions = dimensions;
                chunk.set_title(&format!("window {}", id));
                chunk.keyboard_len.store(0, Ordering::Relaxed);
                chunk.update_frequency = UpdateFrequency::default();

                let target_handle = ChannelHandle::from_parts(sender, self.recv_handle);

                target_handle.send(WindowServerMessage::ConfirmCreation {
                    creation_id,
                    chunk: chunk.clone(),
                });

                self.windows.push(Window {
                    pos: rect.min,
                    chunk,
                    target_handle,
                });

                println!(
                    "created window {} with dimensions {}x{}",
                    id, dimensions.width, dimensions.height
                );
                self.areas_changed = true;

                // syscall::send(
                //     sender,
                //     WindowServerMessage::WindowCreated { id }.into_message(),
                // );
            }

            WindowClientMessage::RequestRender(id) => {
                let window = self.windows.iter_mut().find(|w| w.chunk.id == id);
                if let Some(window) = window {
                    window.chunk.needs_render.store(true, Ordering::Relaxed);
                }
            }
        }
    }

    pub fn draw(
        &mut self,
        fb: &mut Framebuffer,
        input: &mut Input,
        clear_fb: &Framebuffer,
        old_mouse_rect: Rect,
    ) -> RenderResult {
        let mut closed_windows = Vec::new();

        let focused_window = self.windows.len() - 1;

        if input.mouse.left_button.clicked {
            let old_mouse_pos = old_mouse_rect.min;

            let res = self
                .windows
                .iter()
                .enumerate()
                .rev()
                .find_map(|(i, window)| {
                    if window.full_rect().contains(old_mouse_pos) {
                        if window.header_rect().contains(old_mouse_pos) {
                            // focus and drag
                            return Some((i, true));
                        }

                        // focus only
                        return Some((i, false));
                    }

                    None
                });

            if let Some((new_focused_window, drag)) = res {
                self.focus_window(new_focused_window);

                if drag {
                    self.areas_changed = true;
                    self.drag_start = Some(old_mouse_pos);
                }
            }
        }

        if input.mouse.left_button.pressed {
            if let Some(drag_start) = self.drag_start {
                // drag
                let window = &mut self.windows[focused_window];
                let full_rect = window.full_rect();

                let movement = input.mouse.position - drag_start;
                window.pos += movement;

                self.drag_start = Some(input.mouse.position);

                if movement != Position::new(0, 0) {
                    self.areas_changed = true;
                    // let changed_rect_h = if movement.y > 0 {
                    //     Rect::new(
                    //         full_rect.min,
                    //         Position::new(full_rect.max.x, full_rect.min.y + movement.y),
                    //     )
                    // } else {
                    //     Rect::new(
                    //         Position::new(full_rect.min.x, full_rect.max.y),
                    //         full_rect.max - Position::new(0, movement.y),
                    //     )
                    // }
                    // .grow(1); //cover edge cases
                    //
                    // let changed_rect_v = if movement.x > 0 {
                    //     Rect::new(
                    //         full_rect.min,
                    //         Position::new(full_rect.min.x + movement.x, full_rect.max.y),
                    //     )
                    // } else {
                    //     Rect::new(
                    //         Position::new(full_rect.max.x, full_rect.min.y),
                    //         Position::new(full_rect.max.x - movement.x, full_rect.max.y),
                    //     )
                    // }
                    // .grow(1); //cover edge cases

                    fb.clear_region(&full_rect, clear_fb);
                }
            }
        } else {
            self.drag_start = None;
        }

        if self.drag_start.is_some() {
            input.mouse.clear();
            input.mouse.left_button.clicked = false;
        }

        if self.areas_changed {
            self.screen_areas.clear();

            for (i, window) in self.windows.iter().enumerate() {
                let full_rect = window.full_rect();

                // go in reverse so swap_remove doesn't mess up the order
                for i in (0..self.screen_areas.len()).rev() {
                    if let Some(intersection) = self.screen_areas[i]
                        .rect
                        .intersecting_rect(&window.full_rect())
                    {
                        let area = self.screen_areas.swap_remove(i);
                        let rects = [
                            // top
                            Rect::new(
                                area.rect.min,
                                Position::new(area.rect.max.x, intersection.min.y),
                            ),
                            // bottom
                            Rect::new(
                                Position::new(area.rect.min.x, intersection.max.y),
                                area.rect.max,
                            ),
                            // left
                            Rect::new(
                                area.rect.min,
                                Position::new(intersection.min.x, area.rect.max.y),
                            ),
                            // right
                            Rect::new(
                                Position::new(intersection.max.x, area.rect.min.y),
                                area.rect.max,
                            ),
                        ];

                        self.screen_areas
                            .extend(rects.into_iter().filter(|r| r.area() > 0).map(|r| {
                                ScreenArea {
                                    rect: r,
                                    window: area.window,
                                }
                            }));
                    }
                }

                // add new screen area
                self.screen_areas.push(ScreenArea {
                    rect: full_rect,
                    window: i,
                });
            }

            self.screen_areas.sort_unstable_by_key(|a| a.window);
        }

        let mut area_i = 0;

        for (i, window) in self.windows.iter_mut().enumerate() {
            let focused = i == focused_window;
            let mut closed = false;

            if focused
                && self.mouse_grabbed
                && (!window.chunk.grab_mouse || input.keyboard.pressed(KeyCode::LWin))
            {
                self.mouse_grabbed = false;
            }

            let bg_color = if focused {
                Color::new(22, 22, 22)
            } else {
                Color::new(44, 44, 44)
            };

            let full_rect = window.full_rect();

            let window_rect = window.rect();
            let header_rect = window.header_rect();

            while {
                let area = self.screen_areas.get(area_i);
                if let Some(area) = area {
                    area.window == i
                } else {
                    false
                }
            } {
                let area_rect = self.screen_areas[area_i].rect;

                fb.draw_fb_clipped(&window.chunk.fb(), window_rect.min, area_rect);

                area_i += 1;
            }

            fb.draw_box(full_rect, bg_color);
            fb.draw_rect(header_rect, bg_color);

            let mut title_ui = UIFrame::new_stateless(Direction::LeftToRight);
            title_ui.draw_frame(fb, header_rect, input, |ui| {
                ui.margin(MarginMode::Grow);
                ui.label::<font::Cozette>(window.chunk.title());
            });

            let mut btn_ui = UIFrame::new_stateless(Direction::RightToLeft);
            btn_ui.draw_frame(fb, header_rect, input, |ui| {
                ui.margin(MarginMode::Grow);

                if ui.img_button(&self.close_button).clicked {
                    closed = true;
                }
            });

            let should_render = match window.chunk.update_frequency {
                UpdateFrequency::Always => true,
                UpdateFrequency::OnInput => input.any() && focused,
                UpdateFrequency::Manual => false,
            };
            if should_render {
                window.chunk.mouse.position = input.mouse.position - window_rect.min;
                window.chunk.mouse.delta += input.mouse.delta;
                window.chunk.mouse.scroll += input.mouse.scroll;
                window
                    .chunk
                    .mouse
                    .left_button
                    .update(input.mouse.left_button.pressed);
                window
                    .chunk
                    .mouse
                    .right_button
                    .update(input.mouse.right_button.pressed);
                window
                    .chunk
                    .mouse
                    .middle_button
                    .update(input.mouse.middle_button.pressed);

                let current_key_amt = window.chunk.keyboard_len.load(Ordering::Relaxed) as usize;
                let remaining_key_amt = window.chunk.keyboard.len() - current_key_amt;
                let new_key_amt = input.keyboard.keys.len().min(remaining_key_amt);

                let keyboard_src = &input.keyboard.keys[..new_key_amt];
                let keyboard_dest =
                    &mut window.chunk.keyboard[current_key_amt..current_key_amt + new_key_amt];
                keyboard_dest.clone_from_slice(keyboard_src);
                window
                    .chunk
                    .keyboard_len
                    .store((current_key_amt + new_key_amt) as u8, Ordering::Relaxed);

                window.chunk.focused = focused;

                window.chunk.needs_render.store(true, Ordering::Relaxed);
            }

            if closed {
                closed_windows.push(window.chunk.id);
                window
                    .target_handle
                    .send(WindowServerMessage::RequestClose {
                        id: window.chunk.id,
                    });
                self.drag_start = None;

                fb.clear_region(&full_rect, &clear_fb);

                //TODO: free chunk
            }
        }

        let time = syscall::get_time();
        use monos_gfx::text::Font;
        let elapsed = time - self.last_render;
        self.last_render = time;
        if self.debug {
            let fps_rect = Rect::new(
                Position::new(640 - 40, 0),
                Position::new(640, font::Glean::CHAR_HEIGHT as i64),
            );
            fb.clear_region(&fps_rect, clear_fb);
            fb.draw_str::<font::Glean>(
                Color::new(255, 255, 255),
                &format!("{} fps", 1000 / elapsed),
                fps_rect.min,
            );

            for area in &self.screen_areas {
                fb.draw_box(area.rect, Color::new(255, 0, 0));
            }
        }

        self.areas_changed = false;

        if closed_windows.len() > 0 {
            self.windows
                .retain(|w| !closed_windows.contains(&w.chunk.id));
            self.areas_changed = true;
        }

        RenderResult {
            hide_cursor: self.mouse_grabbed,
        }
    }

    pub fn draw_window_list(
        &mut self,
        fb: &mut Framebuffer,
        rect: Rect,
        input: &mut Input,
        clear_fb: &Framebuffer,
    ) {
        fb.clear_region(&rect, clear_fb);

        let mut new_focused_window = None;

        let mut ui = UIFrame::new_stateless(Direction::LeftToRight);
        ui.draw_frame(fb, rect, input, |ui| {
            ui.margin(MarginMode::Grow);

            let mut names = self
                .windows
                .iter()
                .enumerate()
                .map(|(i, w)| (i, w.chunk.id, w.chunk.title()))
                .collect::<Vec<_>>();
            names.sort_by(|a, b| a.1.cmp(&b.1));

            for (i, _, name) in names {
                if ui.button::<font::Cozette>(name).clicked {
                    new_focused_window = Some(i);
                }
            }
        });

        if let Some(new_focused_window) = new_focused_window {
            self.focus_window(new_focused_window);
        }
    }

    fn focus_window(&mut self, new_index: usize) {
        let focused_window = self.windows.len() - 1;
        self.mouse_grabbed = self.windows[new_index].chunk.grab_mouse;

        if new_index == focused_window {
            return;
        }

        self.areas_changed = true;
        self.windows.swap(new_index, focused_window);
    }
}
