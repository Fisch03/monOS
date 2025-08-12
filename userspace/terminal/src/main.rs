#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

mod repl;
mod script;

use monos_gfx::Color;
use monoscript::{ast::Value, ArgArray, Interface, RuntimeErrorKind, ScriptHook};

use monos_std::collections::VecDeque;

enum LineType {
    Input,
    Output,
    Error,
}
impl LineType {
    fn color(&self) -> Color {
        match self {
            LineType::Input => Color::new(255, 255, 255),
            LineType::Output => Color::new(150, 150, 150),
            LineType::Error => Color::new(255, 0, 0),
        }
    }
}

#[derive(Debug, Default)]
struct TerminalInterface {
    lines: VecDeque<String>,
    line_colors: Vec<Color>,
}
impl TerminalInterface {
    fn new() -> Self {
        TerminalInterface {
            lines: VecDeque::new(),
            line_colors: Vec::new(),
        }
    }

    fn add_line(&mut self, line: String, line_type: LineType) {
        let line = match line_type {
            LineType::Input => format!("> {}", line),
            LineType::Error => format!("! {}", line),
            LineType::Output => line,
        };
        self.lines.push_back(line);
        self.line_colors.push(line_type.color());
    }
}
impl<'a> Interface<'a> for TerminalInterface {
    fn inbuilt_function<A: ArgArray<'a>>(
        &mut self,
        ident: &'a str,
        args: A,
    ) -> Result<Value<'a>, RuntimeErrorKind<'a>> {
        match ident {
            "print" => {
                let value = args.get_arg(0, "value")?;
                self.add_line(format!("{value}"), LineType::Output);
                Ok(Value::None)
            }
            "debug" => {
                let value = args.get_arg(0, "value")?;
                self.add_line(format!("{value:?}"), LineType::Output);
                Ok(Value::None)
            }

            "exec" | "run" => {
                let path = args.get_arg(0, "path")?.as_string()?;
                let proc_args = args.get_arg(1, "args").and_then(|a| Ok(a.as_string()?));

                let res = if let Ok(proc_args) = proc_args {
                    syscall::spawn_with_args(path.as_str(), proc_args.as_str())
                } else {
                    syscall::spawn(path.as_str())
                };

                Ok(Value::Number(
                    res.map(|pid| pid.as_u32() as f64).unwrap_or(-1.0),
                ))
            }

            "time" => Ok(Value::Number(syscall::get_time() as f64)),

            "free_mem" => Ok(Value::Number(syscall::sys_info(SysInfo::FreeMemory) as f64)),
            "used_mem" => Ok(Value::Number(syscall::sys_info(SysInfo::UsedMemory) as f64)),
            "total_mem" => Ok(Value::Number(syscall::sys_info(SysInfo::TotalMemory) as f64)),

            "proc_id" => Ok(Value::Number(syscall::sys_info(SysInfo::ProcessId) as f64)),
            "num_proc" => Ok(Value::Number(
                syscall::sys_info(SysInfo::NumProcesses) as f64
            )),

            _ => Err(RuntimeErrorKind::UnknownFunction(ident)),
        }
    }

    fn attach_hook<A: ArgArray<'a>>(
        &mut self,
        kind: &'a str,
        _params: A,
        _hook: ScriptHook<'a>,
    ) -> Result<(), RuntimeErrorKind<'a>> {
        Err(RuntimeErrorKind::UnknownHook(kind))
    }
}

#[no_mangle]
fn main() {
    if args().len() > 0 {
        let path = args()[0].as_str();
        script::run(path)
    } else {
        repl::run()
    };
}
