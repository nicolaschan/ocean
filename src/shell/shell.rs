use std::env;

use pty_process::Size;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Child,
};
use tracing::{debug, error};

use crate::config::model::AppConfig;

use super::event::ShellEvent;

pub fn get_shell(config: &AppConfig) -> String {
    if let Some(shell) = &config.defaults.shell {
        return shell.clone();
    }
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    shell
}

pub async fn spawn_shell(
    config: &AppConfig,
    event_sink: crossbeam::channel::Sender<ShellEvent>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<String>,
) -> Child {
    let pty = pty_process::Pty::new().expect("Failed to create PTY");
    pty.resize(Size::new(100, 100)).unwrap();
    let child = pty_process::Command::new(get_shell(config))
        .spawn(&pty.pts().unwrap())
        .expect("Failed to start shell process");

    let (mut read_pty, mut write_pty) = pty.into_split();

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(input) => {
                    debug!("Reading from user and writing to stdin: {:?}", input);
                    write_pty.write_all(&input.as_bytes()).await.unwrap();
                }
                None => {
                    error!("Nothing to read from user");
                    break;
                }
            }
        }
    });

    tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            debug!("Reading from stdout");
            match read_pty.read(&mut buf).await {
                Ok(0) => {
                    debug!("EOF");
                    break;
                }
                Ok(n) => {
                    event_sink
                        .send(ShellEvent::Output(buf[..n].to_vec()))
                        .expect("Failed to send shell output event");
                }
                Err(e) => {
                    error!("Stdout read failed: {}", e);
                    break;
                }
            }
        }
    });

    child
}
