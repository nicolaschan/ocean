use druid::{AppDelegate, Application, DelegateCtx, Env, Event, KbKey};
use termwiz::escape::{Action, ControlCode};
use tokio::sync::mpsc;
use tracing::{debug, error};

use super::{events::{PROCESS_EXITED, SHELL_OUTPUT}, state::AppState};

pub struct Delegate {
    pub shell_tx: mpsc::UnboundedSender<String>,
}

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppState,
        _env: &druid::widget::prelude::Env,
    ) -> druid::Handled {
        if let Some(_) = cmd.get(PROCESS_EXITED) {
            debug!("Handle shell process exited");
            Application::global().quit();
            return druid::Handled::Yes;
        }
        if let Some(shell_output) = cmd.get(SHELL_OUTPUT) {
            let actions = data
                .parser
                .lock()
                .unwrap()
                .parse_as_vec(&shell_output.as_slice());
            debug!("Shell output (utf8 lossy): {:?}", actions);
            for action in actions {
                match action {
                    Action::Print(c) => {
                        data.shell_string.push(c);
                    }
                    Action::Control(c) => match c {
                        ControlCode::LineFeed => {
                            data.shell_string.push('\n');
                        }
                        ControlCode::CarriageReturn => {
                            // data.shell_string.push('\r');
                        }
                        ControlCode::Backspace => {
                            data.shell_string.pop();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            return druid::Handled::Yes;
        }
        druid::Handled::No
    }

    fn event(
        &mut self,
        _ctx: &mut DelegateCtx,
        _window_id: druid::WindowId,
        event: Event,
        _data: &mut AppState,
        _env: &Env,
    ) -> Option<Event> {
        match event.clone() {
            Event::KeyDown(key_event) => match key_event.key {
                KbKey::Character(c) => {
                    if key_event.mods.ctrl() {
                        if key_event.mods.shift() {
                            match c.chars().next() {
                                Some('V') => {
                                    let copy_paste_contents =
                                        Application::global().clipboard().get_string().unwrap();
                                    if let Err(e) = self.shell_tx.send(copy_paste_contents) {
                                        error!("Error sending key to shell: {}", e);
                                    }
                                    return Some(event.clone());
                                }
                                _ => (),
                            }
                        }
                        match c.chars().next() {
                            Some('c') => {
                                if let Err(e) = self.shell_tx.send("\x03".to_string()) {
                                    error!("Error sending key to shell: {}", e);
                                }
                                return Some(event.clone());
                            }
                            Some('d') => {
                                if let Err(e) = self.shell_tx.send("\x04".to_string()) {
                                    error!("Error sending key to shell: {}", e);
                                }
                                return Some(event.clone());
                            }
                            _ => (),
                        }
                    }
                    if let Err(e) = self.shell_tx.send(c.to_string()) {
                        error!("Error sending key to shell: {}", e);
                    }
                }
                KbKey::Enter => {
                    if let Err(e) = self.shell_tx.send("\n".to_string()) {
                        error!("Error sending key to shell: {}", e);
                    }
                }
                KbKey::Backspace => {
                    if let Err(e) = self.shell_tx.send("\x7f".to_string()) {
                        error!("Error sending key to shell: {}", e);
                    }
                }
                _ => (),
            },
            _ => (),
        }

        Some(event.clone())
    }
}
