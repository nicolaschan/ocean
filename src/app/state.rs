use std::sync::{Arc, Mutex};

use termwiz::{
    escape::parser::Parser,
    surface::{Change, CursorShape, CursorVisibility, Position},
};

use crate::shell::event::ShellEvent;

pub struct TerminalState {
    _cursor_pos: usize,
    pub parser: Arc<Mutex<Parser>>,
    pub surface: termwiz::surface::Surface,
}

impl Default for TerminalState {
    fn default() -> Self {
        let parser = Arc::new(Mutex::new(Parser::new()));
        let surface = termwiz::surface::Surface::new(100, 20);
        Self {
            _cursor_pos: 0,
            parser,
            surface,
        }
    }
}

impl TerminalState {
    pub fn consume(&mut self, event: &ShellEvent) {
        match event {
            ShellEvent::Output(output) => {
                let actions = self.parser.lock().unwrap().parse_as_vec(output.as_slice());
                for action in actions {
                    match action {
                        termwiz::escape::Action::Print(c) => {
                            self.surface
                                .add_change(Change::CursorVisibility(CursorVisibility::Visible));
                            self.surface
                                .add_change(Change::CursorShape(CursorShape::SteadyBar));
                            self.surface.add_change(c);
                        }
                        termwiz::escape::Action::Control(c) => match c {
                            termwiz::escape::ControlCode::LineFeed => {
                                self.surface.add_change('\n');
                            }
                            termwiz::escape::ControlCode::CarriageReturn => {
                                self.surface.add_change('\r');
                            }
                            termwiz::escape::ControlCode::Backspace => {
                                self.surface.add_change(Change::CursorPosition {
                                    x: Position::Relative(-1),
                                    y: Position::Relative(0),
                                });
                                self.surface.add_change(' ');
                                self.surface.add_change(Change::CursorPosition {
                                    x: Position::Relative(-1),
                                    y: Position::Relative(0),
                                });
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

    pub fn get_lines(&self) -> String {
        self.surface.screen_chars_to_string()
    }
}
