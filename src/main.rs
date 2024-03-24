use std::sync::{Arc, Mutex};

use config::model::AppConfig;
use druid::text::{FontDescriptor, FontFamily};
use druid::widget::{Flex, Label, Scroll};
use druid::{
    AppDelegate, AppLauncher, Application, Color, Data, DelegateCtx, Env, Event, ExtEventSink,
    Lens, Target, Widget, WidgetExt, WindowDesc,
};
use druid::{KbKey, UnitPoint};
use shell::{get_shell, SHELL_OUTPUT};
use termwiz::escape::parser::Parser;
use termwiz::escape::{Action, ControlCode};
use tokio::sync::mpsc;
use tracing::{debug, error};

mod config;
mod shell;

pub const PROCESS_EXITED: druid::Selector<()> = druid::Selector::new("process-exited");

fn build_ui(config: &AppConfig) -> impl Widget<AppState> {
    let font_family = FontFamily::new_unchecked(config.font.family.clone());
    let font = FontDescriptor::new(font_family).with_size(config.font.size);

    let label = Label::new(|data: &AppState, _env: &_| data.get_as_string())
        .with_text_size(64.0)
        .with_font(font)
        .with_text_color(Color::WHITE)
        .with_line_break_mode(druid::widget::LineBreaking::WordWrap)
        .align_vertical(UnitPoint::TOP)
        .align_left();
    let label = Scroll::new(label).vertical().expand_height();

    Flex::column()
        .with_flex_child(label, 1.0)
        .background(Color::rgba8(
            0,
            0,
            0,
            (255.0 - config.window.transparency * 255.0 / 100.0) as u8,
        ))
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    shell_string: String,
    cursor_pos: usize,
    #[data(eq)]
    shell_output: Vec<u8>,
    #[data(ignore)]
    parser: Arc<Mutex<Parser>>,
}

impl AppState {
    pub fn get_as_string(&self) -> String {
        self.shell_string.clone() + "█"
        // String::from_utf8_lossy(&self.shell_output).to_string()
    }
}

pub struct Delegate {
    shell_tx: mpsc::UnboundedSender<String>,
    _event_sink: ExtEventSink,
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
            data.shell_output
                .extend_from_slice(&shell_output.as_slice());
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::config::read_config().expect("Failed to read config file");

    let (shell_tx, shell_rx) = mpsc::unbounded_channel();

    let main_window = WindowDesc::new(build_ui(&config))
        .title(format!("{} — {}", get_shell(&config), config.window.title))
        .transparent(true)
        .window_size((config.window.width, config.window.height));

    let launcher = AppLauncher::with_window(main_window);
    let event_sink = launcher.get_external_handle();
    let delegate = Delegate {
        shell_tx,
        _event_sink: event_sink.clone(),
    };

    let app_state = AppState {
        cursor_pos: 0,
        shell_string: String::new(),
        shell_output: Vec::new(),
        parser: Arc::new(Mutex::new(Parser::new())),
    };

    tokio::spawn(async move {
        let mut child = shell::spawn_shell(&config, event_sink.clone(), shell_rx).await;
        let _ = child
            .wait()
            .await
            .expect("Failed to wait for shell process");
        debug!("Shell process exited");
        event_sink
            .submit_command(PROCESS_EXITED, (), Target::Auto)
            .unwrap();
    });

    launcher
        .delegate(delegate)
        .launch(app_state)
        .expect("Failed to launch application");
}
