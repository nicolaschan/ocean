use std::sync::{Arc, Mutex};

use druid::{Data, Lens};
use termwiz::escape::parser::Parser;

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub shell_string: String,
    cursor_pos: usize,
    #[data(ignore)]
    pub parser: Arc<Mutex<Parser>>,
}

impl AppState {
    pub fn new() -> Self {
        let parser = Arc::new(Mutex::new(Parser::new()));
        Self {
            shell_string: String::new(),
            cursor_pos: 0,
            parser,
        }
    }

    pub fn get_as_string(&self) -> String {
        self.shell_string.clone() + "█"
    }
}
