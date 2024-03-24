use app::delegate::Delegate;
use app::state::AppState;
use config::model::AppConfig;
use druid::text::{FontDescriptor, FontFamily};
use druid::widget::{Flex, Label, Scroll};
use druid::UnitPoint;
use druid::{AppLauncher, Color, Target, Widget, WidgetExt, WindowDesc};
use shell::get_shell;
use tokio::sync::mpsc;
use tracing::debug;

use crate::app::events::PROCESS_EXITED;

mod app;
mod config;
mod shell;

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
    let delegate = Delegate { shell_tx };

    let app_state = AppState::new();

    let shell = tokio::spawn(async move {
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

    shell.await.unwrap();
}
