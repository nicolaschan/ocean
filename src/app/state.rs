use std::sync::{Arc, Mutex};

use termwiz::escape::parser::Parser;

use crate::shell::event::ShellEvent;

pub struct TerminalState {
    pub shell_string: String,
    _cursor_pos: usize,
    pub parser: Arc<Mutex<Parser>>,
}

impl TerminalState {
    pub fn new() -> Self {
        let parser = Arc::new(Mutex::new(Parser::new()));
        Self {
            shell_string: String::new(),
            _cursor_pos: 0,
            parser,
        }
    }

    pub fn consume(&mut self, event: ShellEvent) {
        match event {
            ShellEvent::Output(output) => {
                let actions = self.parser.lock().unwrap().parse_as_vec(&output.as_slice());
                for action in actions {
                    match action {
                        termwiz::escape::Action::Print(c) => {
                            self.shell_string.push(c);
                        }
                        termwiz::escape::Action::Control(c) => match c {
                            termwiz::escape::ControlCode::LineFeed => {
                                self.shell_string.push('\n');
                            }
                            termwiz::escape::ControlCode::CarriageReturn => {
                                // self.shell_string.push('\r');
                            }
                            termwiz::escape::ControlCode::Backspace => {
                                self.shell_string.pop();
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            ShellEvent::ProcessExited => {}
        }
    }

    pub fn get_as_string(&self) -> String {
        self.shell_string.clone() + "█"
    }
}
