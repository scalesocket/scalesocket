use {
    std::collections::HashMap,
    std::env,
    std::process::{ExitStatus, Stdio},
    tokio::process::Command,
};

use crate::types::PortID;

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
    use std::{collections::HashMap, convert::Infallible};

    use futures::future::ready;
    use warp::{self, http::StatusCode, reject::Reject, Filter, Rejection, Reply};

    use crate::envvars::{CGIEnv, Env};

    pub type One<T> = (T,);

    #[derive(Debug)]
    pub struct InvalidRoom;
    impl Reject for InvalidRoom {}

    pub fn enable_if(condition: bool) -> impl Filter<Extract = (), Error = Rejection> + Copy {
        warp::any()
            .and_then(move || {
                if condition {
                   ready(Ok(()))
                } else {
                   ready(Err(warp::reject::not_found()))
                }
            })
            // deal with Ok(())
            .untuple_one()
    }

    pub fn env() -> impl Filter<Extract = One<Env>, Error = Rejection> + Copy {
        warp::any()
            .and(cgi_env())
            .and(warp::query::<HashMap<String, String>>())
            .and_then(move |cgi, query| ready(Ok::<_, Rejection>((Env { cgi, query },))))
            // deal with Ok(())
            .untuple_one()
    }

    pub fn cgi_env() -> impl Filter<Extract = One<CGIEnv>, Error = Rejection> + Copy {
        let optional_query = warp::query::raw()
            .map(Some)
            .or_else(|_| async { Ok::<One<Option<String>>, Infallible>((None,)) });

        warp::any()
            .and(optional_query)
            .and(warp::addr::remote())
            .and_then(move |query, addr| {
                ready(Ok::<_, Rejection>((CGIEnv::from_filter(query, addr),)))
            })
            // deal with Ok(())
            .untuple_one()
    }

    pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
        use warp::reply::with_status as status;
        if err.is_not_found() {
            Ok(status("Not found", StatusCode::NOT_FOUND))
        } else if err.find::<InvalidRoom>().is_some() {
            Ok(status("Invalid room", StatusCode::BAD_REQUEST))
        } else {
            Ok(status("Internal error", StatusCode::INTERNAL_SERVER_ERROR))
        }
    }

    pub mod path {

        use std::{ops::Deref, str::FromStr};

        use super::*;
        pub fn param_not<T: FromStr + Send + Deref<Target = str> + 'static>(
            denylist: &'static [&'static str],
        ) -> impl Filter<Extract = One<T>, Error = Rejection> + Clone {
            warp::path::param::<T>()
                .and_then(move |param: T| {
                    if denylist.contains(&param.deref()) {
                        ready(Err(warp::reject::custom(InvalidRoom)))
                    } else {
                        ready(Ok::<_, Rejection>((param,)))
                    }
                })
                // deal with Ok(())
                .untuple_one()
        }
    }
}
