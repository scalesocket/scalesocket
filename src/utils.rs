use crate::types::{ConnID, PortID};
use {
    std::collections::HashMap,
    std::env,
    std::process::{ExitStatus, Stdio},
    std::sync::atomic::{AtomicU32, Ordering},
    tokio::process::Command,
};

/// Our global unique connection id counter.
static NEXT_CONNECTION_ID: AtomicU32 = AtomicU32::new(1);

#[allow(unreachable_code)]
pub fn new_conn_id() -> ConnID {
    #[cfg(test)]
    return 1;

    NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn run(
    program: &str,
    args: Vec<String>,
    port: Option<PortID>,
    env_extra: HashMap<String, String>,
    env_allowlist: &[String],
) -> Command {
    // Combine filtered environment with external variables
    let env: HashMap<String, String> = env::vars()
        .filter(|(k, _)| env_allowlist.contains(k))
        .chain(env_extra)
        .collect();

    let mut cmd = Command::new(program);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(args)
        .kill_on_drop(true)
        .env_clear()
        .envs(&env);

    if let Some(port) = port {
        cmd.env("PORT", port.to_string());
    }
    cmd
}

pub fn exit_code<T>(status: Result<ExitStatus, T>) -> Option<i32> {
    status.ok().and_then(|s| s.code()).or(None)
}

/// Utility filters for Warp
pub mod warpext {
    use crate::envvars::{CGIEnv, Env};
    use std::collections::HashMap;
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

    pub fn env() -> impl Filter<Extract = One<Env>, Error = Rejection> + Copy {
        warp::any()
            .and(cgi_env())
            .and(warp::query::<HashMap<String, String>>())
            .and_then(async move |cgi, query| Ok::<_, Rejection>((Env { cgi, query },)))
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
