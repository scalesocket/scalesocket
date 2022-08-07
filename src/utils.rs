use crate::types::{ConnID, PortID};
use {
    std::collections::HashMap,
    std::env,
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

pub fn run<I, S>(
    program: &str,
    args: I,
    port: Option<PortID>,
    env_extra: HashMap<String, String>,
) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::new(program);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(args)
        .kill_on_drop(true)
        .env_clear()
        .envs(&env_extra);

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
    use crate::types::CGIEnv;
    use warp::{self, Filter, Rejection};

    pub type One<T> = (T,);

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

    pub fn cgi_env() -> impl Filter<Extract = One<CGIEnv>, Error = Rejection> + Copy {
        let optional_query = warp::query::raw()
            .map(Some)
            .or_else(|_| async { Ok::<One<Option<String>>, std::convert::Infallible>((None,)) });

        warp::any()
            .and(optional_query)
            .and(warp::addr::remote())
            .and_then(async move |query, addr| {
                Ok::<_, Rejection>((CGIEnv::from_filter(query, addr),))
            })
            // deal with Ok(())
            .untuple_one()
    }
}
