use crate::types::{ConnID, PortID};
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

pub fn run<I, S>(program: &str, args: I, port: Option<PortID>) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::new(program);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(args)
        .kill_on_drop(true)
        .env_clear();

    if let Some(port) = port {
        cmd.env("PORT", port.to_string());
    }
    cmd
}

pub fn exit_code<T>(status: Result<ExitStatus, T>) -> Option<i32> {
    status.ok().and_then(|s| s.code()).or(None)
}

// utility filters for warp
pub mod warpext {
    use warp::{self, Filter, Rejection};

    pub fn enable_if(condition: bool) -> impl Filter<Extract = (), Error = Rejection> + Copy {
        warp::any()
            .and_then(async move || {
                if condition {
                    Ok(())
                } else {
                    Err(warp::reject::not_found())
                }
            })
            // deal with Ok(())
            .untuple_one()
    }
}
