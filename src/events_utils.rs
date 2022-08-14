use crate::types::Env;
use {std::collections::HashMap, urlencoding::encode};

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

    use crate::types::{CGIEnv, Env};

    use super::replace_template_env;

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
