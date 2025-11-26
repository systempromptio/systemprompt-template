use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub struct TemplateEngine;

impl TemplateEngine {
    pub fn render(template: &str, context: HashMap<&str, &str>) -> String {
        let mut result = template.to_string();

        for (key, value) in context {
            let placeholder = format!("{{{key}}}");
            let escaped_value = Self::html_escape(value);
            result = result.replace(&placeholder, &escaped_value);
        }

        result
    }

    fn html_escape(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    pub const fn load_authorize_template() -> &'static str {
        include_str!("../templates/authorize.html")
    }

    pub const fn load_webauthn_oauth_template() -> &'static str {
        include_str!("../templates/webauthn_oauth.html")
    }
}
