use monos_gfx::{
    font::{self, Font},
    text::Origin,
    ui::{widgets, Direction, TextWrap, UIFrame},
    Color, Dimension, Input, Position, Rect,
};
use monos_std::collections::VecDeque;
use monoscript::{ast::Value, Interface, ScriptContext, ScriptHook};
use rooftop::{UpdateFrequency, Window, WindowClient, WindowHandle};

use super::{LineType, TerminalInterface};

const WINDOW_LIMIT: usize = 16;

struct ScriptState<'a> {
    context: Option<ScriptContext<'a>>,
    interface: ScriptInterface<'a>,
    ui: UIFrame,
}

impl<'a> ScriptState<'a> {
    pub fn new(context: Option<ScriptContext<'a>>, interface: ScriptInterface<'a>) -> Self {
        Self {
            context,
            interface,
            ui: UIFrame::new(Direction::BottomToTop),
        }
    }
}

struct ScriptWindow<'a> {
    title: String,
    hook: ScriptHook<'a>,
    handle: WindowHandle,
}

struct ScriptInterface<'a> {
    terminal: TerminalInterface,
    rendered_lines: usize,
    windows: Vec<ScriptWindow<'a>>,
    window_queue: VecDeque<(String, ScriptHook<'a>)>,
}

impl ScriptInterface<'_> {
    fn new() -> Self {
        Self {
            terminal: TerminalInterface::new(),
            rendered_lines: 0,
            windows: Vec::new(),
            window_queue: VecDeque::new(),
        }
    }
}

impl<'a> Interface<'a> for ScriptInterface<'a> {
    fn inbuilt_function<A: monoscript::ArgArray<'a>>(
        &mut self,
        ident: &'a str,
        args: A,
    ) -> Result<monoscript::ast::Value<'a>, monoscript::RuntimeErrorKind<'a>> {
        match ident {
            _ => self.terminal.inbuilt_function(ident, args),
        }
    }

    fn attach_hook<A: monoscript::ArgArray<'a>>(
        &mut self,
        kind: &'a str,
        params: A,
        hook: ScriptHook<'a>,
    ) -> Result<(), monoscript::RuntimeErrorKind<'a>> {
        match kind {
            "window" => {
                let title = params
                    .get_arg(0, "window title")
                    .and_then(Value::as_string)
                    .unwrap_or(String::from("monoscript window"));

                self.window_queue.push_back((title, hook));
                Ok(())
            }

            _ => self.terminal.attach_hook(kind, params, hook),
        }
    }
}

impl<'a> core::ops::Deref for ScriptInterface<'a> {
    type Target = TerminalInterface;
    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}
impl<'a> core::ops::DerefMut for ScriptInterface<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

struct WindowInterface<'w, 'a, 'fb> {
    interface: &'w mut ScriptInterface<'a>,
    window: &'w mut Window<'fb>,
    color: Color,
}

impl<'a> Interface<'a> for WindowInterface<'_, 'a, '_> {
    fn inbuilt_function<A: monoscript::ArgArray<'a>>(
        &mut self,
        ident: &'a str,
        args: A,
    ) -> Result<monoscript::ast::Value<'a>, monoscript::RuntimeErrorKind<'a>> {
        match ident {
            "box" => {
                let x = args.get_arg(0, "box x position")?.as_number()? as i64;
                let y = args.get_arg(1, "box y position")?.as_number()? as i64;
                let w = args.get_arg(2, "box width")?.as_number()? as i64;
                let h = args.get_arg(3, "box height")?.as_number()? as i64;

                self.window.draw_rect(
                    Rect::new(Position::new(x, y), Position::new(x + w, y + h)),
                    self.color,
                );

                Ok(Value::None)
            }
            "square" => {
                let x = args.get_arg(0, "square x position")?.as_number()? as i64;
                let y = args.get_arg(1, "square y position")?.as_number()? as i64;
                let s = args.get_arg(2, "square size")?.as_number()? as i64;

                self.window.draw_rect(
                    Rect::new(Position::new(x, y), Position::new(x + s, y + s)),
                    self.color,
                );

                Ok(Value::None)
            }

            "color" => {
                let r = args.get_arg(0, "red")?.as_number()? as u8;
                let g = args.get_arg(1, "green")?.as_number()? as u8;
                let b = args.get_arg(2, "blue")?.as_number()? as u8;
                self.color = Color::new(r, g, b);
                Ok(Value::None)
            }

            _ => self.interface.inbuilt_function(ident, args),
        }
    }

    fn attach_hook<A: monoscript::ArgArray<'a>>(
        &mut self,
        kind: &'a str,
        params: A,
        hook: ScriptHook<'a>,
    ) -> Result<(), monoscript::RuntimeErrorKind<'a>> {
        match kind {
            _ => self.interface.attach_hook(kind, params, hook),
        }
    }
}

impl<'a> core::ops::Deref for WindowInterface<'_, 'a, '_> {
    type Target = ScriptInterface<'a>;
    fn deref(&self) -> &Self::Target {
        &self.interface
    }
}

impl<'a> core::ops::DerefMut for WindowInterface<'_, 'a, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.interface
    }
}

pub fn run<'p, P: Into<Path<'p>>>(path: P) -> ! {
    let path = path.into();
    let window_title = path.file_name().unwrap_or(path.as_str()).to_string();

    let file = File::open(path).expect("failed to open script file");
    let script = file.read_to_string().expect("failed to read script file");

    let mut interface = ScriptInterface::new();

    let parsed = monoscript::parse(&script).expect("failed to parse script");
    let res = monoscript::execute(parsed, &mut interface);

    match res {
        Ok(_) => {}
        Err(ref err) => {
            interface.add_line(format!("{:?}", err), LineType::Error);
        }
    }

    let open_terminal = interface.windows.is_empty()
        && interface.window_queue.is_empty()
        && interface.lines.is_empty();

    let mut window_client =
        WindowClient::new("desktop.windows", ScriptState::new(res.ok(), interface)).unwrap();

    let mut terminal_window = if open_terminal {
        Some(window_client.create_window(&window_title, Dimension::new(320, 240), render))
    } else {
        None
    };

    loop {
        if !window_client.data().interface.window_queue.is_empty() {
            let new_windows = window_client
                .data_mut()
                .interface
                .window_queue
                .drain(..)
                .collect::<Vec<_>>();

            if window_client.data().interface.windows.len() + new_windows.len() > WINDOW_LIMIT {
                window_client
                    .data_mut()
                    .interface
                    .add_line(String::from("window limit reached"), LineType::Error);
                continue;
            }

            let new_windows = new_windows
                .into_iter()
                .map(|(title, hook)| {
                    let handle = window_client.next_handle();
                    window_client.create_window(
                        &title,
                        Dimension::new(320, 240),
                        move |window, state, _| {
                            *window.update_frequency = UpdateFrequency::Always;

                            let script_window = state
                                .interface
                                .windows
                                .iter()
                                .position(|w| w.handle == handle)
                                .unwrap();

                            let script_window = state.interface.windows.swap_remove(script_window);

                            let mut interface = WindowInterface {
                                interface: &mut state.interface,
                                window,
                                color: Color::new(255, 255, 255),
                            };

                            let res = script_window
                                .hook
                                .execute(state.context.as_mut().unwrap(), &mut interface);

                            match res {
                                Ok(_) => {}
                                Err(ref err) => {
                                    interface.add_line(format!("{:?}", err), LineType::Error);
                                }
                            }

                            state.interface.windows.push(script_window);
                        },
                    );
                    ScriptWindow {
                        title,
                        hook,
                        handle,
                    }
                })
                .collect::<Vec<_>>();

            window_client
                .data_mut()
                .interface
                .windows
                .extend(new_windows);
        }

        window_client.update();

        if let Some(term) = terminal_window {
            if window_client.data().interface.rendered_lines
                != window_client.data().interface.lines.len()
            {
                window_client.request_render(term);
            }
        } else if !window_client.data().interface.lines.is_empty() {
            terminal_window =
                Some(window_client.create_window(&window_title, Dimension::new(320, 240), render));
        }

        syscall::yield_();
    }
}

fn render(window: &mut Window, state: &mut ScriptState, mut input: Input) {
    *window.update_frequency = UpdateFrequency::Manual;

    window.clear();

    let rect = Rect::from_dimensions(window.dimensions()).shrink(2);

    state.ui.draw_frame(window, rect, &mut input, |ui| {
        ui.gap(0);

        ui.add(
            widgets::ScrollableLabel::<font::Glean, _>::new_iter(
                state.interface.lines.iter().map(|line| line.as_str()),
                Origin::Bottom,
            )
            .wrap(TextWrap::Enabled { hyphenate: false })
            .scroll_y(rect.height() - font::Glean::CHAR_HEIGHT - 4)
            .text_colors(state.interface.line_colors.as_slice()),
        );
    });

    state.interface.rendered_lines = state.interface.lines.len();
}
