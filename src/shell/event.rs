#[derive(PartialEq)]
pub enum ShellEvent {
    ProcessExited,
    Output(Vec<u8>),
}
