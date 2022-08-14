use {std::collections::HashMap, std::net::SocketAddr, urlencoding::encode};

#[derive(Clone, Debug, Default)]
pub struct Env {
    pub cgi: CGIEnv,
    pub query: HashMap<String, String>,
}

impl From<Env> for HashMap<String, String> {
    /// Performs conversion and turns query keys to uppercase
    fn from(env: Env) -> Self {
        let mut result: HashMap<String, String> = env.cgi.into();
        let query_uppercase: HashMap<String, String> = env
            .query
            .into_iter()
            .map(|(k, v)| (k.to_uppercase(), v))
            .collect();

        result.extend(query_uppercase);
        result
    }
}

#[derive(Clone, Debug, Default)]
pub struct CGIEnv {
    /// URL-encoded search or parameter string
    query_string: String,
    /// network address of the client sending the request
    remote_addr: String,
}

impl CGIEnv {
    pub fn from_filter(query_string: Option<String>, remote_addr: Option<SocketAddr>) -> Self {
        let query_string = query_string.unwrap_or_default();
        let remote_addr = remote_addr
            .map(|a| a.to_string())
            .unwrap_or_else(|| "".to_string());

        Self {
            query_string,
            remote_addr,
        }
    }
}

impl From<CGIEnv> for HashMap<String, String> {
    fn from(env: CGIEnv) -> Self {
        HashMap::from([
            // NOTE: implicit uppercase
            ("QUERY_STRING".to_string(), env.query_string),
            ("REMOTE_ADDR".to_string(), env.remote_addr),
        ])
    }
}

pub fn replace_template_env(template: &str, conn: usize, env: &Env) -> String {
    let template = template.replace("#ID", &conn.to_string());

    // NOTE: implicit uppercase
    let cgi_upcase_keys = env.cgi.clone().into();
    let query_upcase_keys = env.query.clone().keys_upper();

    let result = replace_template(template, query_upcase_keys, "#QUERY_", true);
    replace_template(result, cgi_upcase_keys, "#", false)
}

pub fn replace_template(
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

    #[tokio::test]
    async fn test_replace_template_env() {
        let env = Env {
            cgi: CGIEnv::default(),
            query: create_query(),
        };
        let conn = 1;
        let result = replace_template_env("test #ID #QUERY_FOO", conn, &env);

        assert_eq!(result, "test 1 bar%20baz");
    }
}
