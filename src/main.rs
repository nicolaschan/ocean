use std::{collections::HashMap, thread};

#[cfg(target_os = "macos")]
use bevy::window::CompositeAlphaMode;
use bevy::{
    app::AppExit,
    prelude::*,
    window::{Cursor, PresentMode},
};
use ocean::{
    app::state::TerminalState,
    config::config,
    shell::{event::ShellEvent, shell},
};
use once_cell::sync::Lazy;
use tokio::sync::mpsc;

#[derive(Event)]
struct UpdateEvent(ShellEvent);

#[derive(Component)]
struct Terminal {
    terminal_state: TerminalState,
}

#[derive(Resource)]
struct ShellSender(mpsc::UnboundedSender<String>);

fn add_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let config = config::read_config().unwrap();

    commands.spawn(Camera2dBundle::default());
    commands.spawn(TextBundle {
        text: Text::from_section(
            "Filling the ocean... 🌊",
            TextStyle {
                font: asset_server.load(config.font.family),
                font_size: config.font.size,
                color: Color::WHITE,
            },
        ),
        ..default()
    });
}

fn handle_update_event(
    mut events: EventReader<UpdateEvent>,
    mut terminals: Query<&mut Terminal>,
    mut exit: EventWriter<AppExit>,
) {
    for event in events.read().into_iter() {
        if event.0 == ShellEvent::ProcessExited {
            debug!("Shell process exited");
            exit.send(AppExit);
            return;
        }
        for mut terminal in terminals.iter_mut() {
            debug!("Consuming update event: {:?}", event.0);
            terminal.terminal_state.consume(&event.0);
        }
    }
}

fn handle_received_characters(
    mut events: EventReader<ReceivedCharacter>,
    keyboard: Res<ButtonInput<KeyCode>>,
    shell_tx: ResMut<ShellSender>,
) {
    if keyboard.any_pressed([KeyCode::ControlLeft]) {
        return;
    }
    for event in events.read() {
        shell_tx
            .0
            .send(event.char.to_string())
            .expect("Failed to send character to shell");
    }
}

static CTRL_KEY_MAPPING: Lazy<HashMap<String, String>> = Lazy::new(|| {
    [
        ("c".to_string(), "\u{3}".to_string()),
        ("d".to_string(), "\u{4}".to_string()),
    ]
    .iter()
    .cloned()
    .collect()
});

fn handle_key_chords(
    mut events: EventReader<ReceivedCharacter>,
    keyboard: Res<ButtonInput<KeyCode>>,
    shell_tx: ResMut<ShellSender>,
) {
    if !keyboard.any_pressed([KeyCode::ControlLeft]) {
        return;
    }
    for event in events.read() {
        let ctrl_mapping = CTRL_KEY_MAPPING.get(&event.char.to_string());
        if let Some(ctrl_mapping) = ctrl_mapping {
            shell_tx
                .0
                .send(ctrl_mapping.clone())
                .expect("Failed to send character to shell");
            return;
        }
    }
}

fn redraw_terminal(terminals: Query<&Terminal>, mut text: Query<&mut Text>) {
    for terminal in terminals.iter() {
        for mut text in text.iter_mut() {
            text.sections[0].value = terminal.terminal_state.get_lines(20);
        }
    }
}

fn main() {
    let config = config::read_config().unwrap();

    let window_title = format!("{} — {}", shell::get_shell(&config), config.window.title);
    let background_color = ClearColor(Color::rgba(0.0, 0.0, 0.0, config.window.transparency));

    let (event_sink, shell_event_receiver) = crossbeam::channel::unbounded();
    let (shell_tx, shell_rx) = mpsc::unbounded_channel();

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime");

        runtime.block_on(async move {
            let mut child = shell::spawn_shell(&config, event_sink.clone(), shell_rx).await;
            let _ = child
                .wait()
                .await
                .expect("Failed to wait for shell process");
            debug!("Shell process exited");
            event_sink
                .send(ShellEvent::ProcessExited)
                .expect("Failed to send shell event");
        });
    });

    App::new()
        .insert_resource(background_color)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Fifo,
                transparent: true,
                title: window_title,
                cursor: Cursor::default(),
                #[cfg(target_os = "macos")]
                composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ShellSender(shell_tx))
        .add_systems(Startup, move |mut commands: Commands| {
            commands.spawn(Terminal {
                terminal_state: TerminalState::new(),
            });
        })
        .add_systems(Startup, add_text)
        .add_event::<UpdateEvent>()
        .add_systems(Update, move |mut events: EventWriter<UpdateEvent>| {
            if let Ok(event) = shell_event_receiver.try_recv() {
                events.send(UpdateEvent(event));
            }
        })
        .add_systems(Update, handle_update_event)
        .add_systems(Update, redraw_terminal)
        .add_systems(Update, (handle_received_characters, handle_key_chords))
        .run();
}
