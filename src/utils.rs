use crate::types::ConnID;
use {
    std::process::ExitStatus,
    std::process::Stdio,
    std::sync::atomic::{AtomicUsize, Ordering},
    tokio::process::Command,
};

/// Our global unique connection id counter.
static NEXT_CONNECTION_ID: AtomicUsize = AtomicUsize::new(1);

pub fn new_conn_id() -> ConnID {
    NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn run<I, S>(program: &str, args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::new(program);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(args)
        .kill_on_drop(true);
    cmd
}

pub fn exit_code<T>(status: Result<ExitStatus, T>) -> Option<i32> {
    status.ok().and_then(|s| s.code()).or(None)
}
