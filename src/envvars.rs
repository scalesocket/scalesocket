use crate::types::ConnID;
use {std::collections::HashMap, std::net::SocketAddr, urlencoding::encode};

#[derive(Clone, Debug, Default)]
pub struct Env {
    pub cgi: CGIEnv,
    pub query: HashMap<String, String>,
}

impl Env {
    pub fn set_room(&mut self, room: &str) -> &mut Self {
        self.cgi.room = room.to_string();
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct CGIEnv {
    /// URL-encoded search or parameter string
    query_string: String,
    /// network address of the client sending the request
    remote_addr: String,
    /// room name (non standard)
    room: String,
}

impl CGIEnv {
    pub fn from_filter(query_string: Option<String>, remote_addr: Option<SocketAddr>) -> Self {
        let query_string = query_string.unwrap_or_default();
        let remote_addr = remote_addr.map(|a| a.to_string()).unwrap_or_default();

        Self {
            query_string,
            remote_addr,
            ..Default::default()
        }
    }
}

impl From<CGIEnv> for HashMap<String, String> {
    fn from(env: CGIEnv) -> Self {
        HashMap::from([
            // NOTE: implicit uppercase
            ("QUERY_STRING".to_string(), env.query_string),
            ("REMOTE_ADDR".to_string(), env.remote_addr),
            ("ROOM".to_string(), env.room),
        ])
    }
}

pub fn replace_template_env(template: &str, conn: ConnID, env: &Env) -> String {
    let template = template.replace("#ID", &conn.to_string());

    let cgi_vars = env.cgi.clone().into();
    let query_vars = env.query.clone().keys_upper();

    let result = replace_template(template, cgi_vars, "#", false);
    replace_template(result, query_vars, "#QUERY_", true)
}

fn replace_template(
    template: String,
    replace_values: HashMap<String, String>,
    prefix: &str,
    urlencode: bool,
) -> String {
    let mut result = template;
    for (key, value) in replace_values {
        let variable = &[prefix, &key].concat();
        if urlencode {
            result = result.replace(variable, &encode(&value));
        } else {
            result = result.replace(variable, &value);
        };
    }
    result
}
trait HashMapExt {
    fn keys_upper(self) -> HashMap<String, String>;
}

impl HashMapExt for HashMap<String, String> {
    fn keys_upper(self) -> HashMap<String, String> {
        self.into_iter()
            .map(|(k, v)| (k.to_uppercase(), v))
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::{replace_template_env, CGIEnv, Env};

    fn create_query() -> HashMap<String, String> {
        HashMap::from([("foo".to_string(), "bar baz".to_string())])
    }

    fn create_cgi() -> CGIEnv {
        CGIEnv {
            query_string: "foo=".to_string(),
            remote_addr: "127.0.0.1:1234".to_string(),
            room: "room".to_string(),
        }
    }

    #[tokio::test]
    async fn test_replace_template_env() {
        let env = Env {
            cgi: create_cgi(),
            query: create_query(),
        };
        let result = replace_template_env("test #ID #REMOTE_ADDR #QUERY_FOO", 1, &env);

        assert_eq!(result, "test 1 127.0.0.1:1234 bar%20baz");
    }

    #[tokio::test]
    async fn test_replace_template_env_omits_query_overrides() {
        let query = HashMap::from([
            ("remote_addr".to_string(), "overridden".to_string()),
            ("string".to_string(), "overridden".to_string()),
            ("hack".to_string(), "#SOMETHING".to_string()),
        ]);
        let env = Env {
            cgi: create_cgi(),
            query,
        };
        let result = replace_template_env("test #REMOTE_ADDR #QUERY_STRING #QUERY_HACK", 1, &env);

        assert_eq!(result, "test 127.0.0.1:1234 foo= %23SOMETHING");
    }
}
