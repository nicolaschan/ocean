use std::env;

use druid::{ExtEventSink, Target};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Child,
    sync::mpsc,
};
use tracing::{debug, error};

use crate::config::model::AppConfig;

pub const SHELL_OUTPUT: druid::Selector<Vec<u8>> = druid::Selector::new("shell-output");

pub fn get_shell(config: &AppConfig) -> String {
    if let Some(shell) = &config.defaults.shell {
        return shell.clone();
    }
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    shell
}

pub async fn spawn_shell(
    config: &AppConfig,
    event_sink: ExtEventSink,
    mut rx: mpsc::UnboundedReceiver<String>,
) -> Child {
    let pty = pty_process::Pty::new().expect("Failed to create PTY");
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
                        .submit_command(SHELL_OUTPUT, buf[..n].to_vec(), Target::Auto)
                        .expect("Failed to send shell output to UI");
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
