use rand::seq::SliceRandom;
use reqwest::header::HeaderMap;
use serde_json::json;
use uuid::Uuid;

pub fn user_agent() -> String {
    let mut rng = rand::thread_rng();
    vec![
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
    ]
    .choose(&mut rng)
    .unwrap()
    .to_string()
}

pub fn fingerprint() -> String {
    format!("test-{}", Uuid::new_v4())
}

pub fn conversation_message(content: &str) -> serde_json::Value {
    json!({
        "messages": [
            {
                "role": "user",
                "content": content
            }
        ]
    })
}

pub struct SessionFactory {
    pub user_agent: String,
    pub ip: String,
    pub fingerprint: String,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
}

impl Default for SessionFactory {
    fn default() -> Self {
        Self {
            user_agent: user_agent(),
            ip: "8.8.8.8".to_string(),
            fingerprint: fingerprint(),
            utm_source: None,
            utm_medium: None,
        }
    }
}

impl SessionFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_utm(mut self, source: &str, medium: &str) -> Self {
        self.utm_source = Some(source.to_string());
        self.utm_medium = Some(medium.to_string());
        self
    }

    pub fn with_ip(mut self, ip: &str) -> Self {
        self.ip = ip.to_string();
        self
    }

    pub fn with_fingerprint(mut self, fingerprint: &str) -> Self {
        self.fingerprint = fingerprint.to_string();
        self
    }

    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = user_agent.to_string();
        self
    }

    pub fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", self.user_agent.parse().unwrap());
        headers.insert("X-Forwarded-For", self.ip.parse().unwrap());
        headers.insert("X-Fingerprint-Hash", self.fingerprint.parse().unwrap());

        if let Some(utm_source) = &self.utm_source {
            headers.insert("X-UTM-Source", utm_source.parse().unwrap());
        }
        if let Some(utm_medium) = &self.utm_medium {
            headers.insert("X-UTM-Medium", utm_medium.parse().unwrap());
        }

        headers
    }
}

pub struct ConversationFactory {
    pub messages: Vec<serde_json::Value>,
}

impl Default for ConversationFactory {
    fn default() -> Self {
        Self { messages: vec![] }
    }
}

impl ConversationFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_message(mut self, role: &str, content: &str) -> Self {
        self.messages.push(json!({
            "role": role,
            "content": content
        }));
        self
    }

    pub fn build(&self) -> serde_json::Value {
        json!({
            "messages": self.messages
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_is_unique() {
        let fp1 = fingerprint();
        let fp2 = fingerprint();
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_session_factory_defaults() {
        let factory = SessionFactory::new();
        assert!(!factory.user_agent.is_empty());
        assert!(!factory.fingerprint.is_empty());
        assert_eq!(factory.ip, "8.8.8.8");
    }

    #[test]
    fn test_session_factory_builder() {
        let factory = SessionFactory::new()
            .with_utm("google", "cpc")
            .with_ip("1.2.3.4");

        assert_eq!(factory.utm_source, Some("google".to_string()));
        assert_eq!(factory.utm_medium, Some("cpc".to_string()));
        assert_eq!(factory.ip, "1.2.3.4");
    }

    #[test]
    fn test_conversation_factory() {
        let conv = ConversationFactory::new()
            .with_message("user", "Hello")
            .with_message("assistant", "Hi there")
            .build();

        assert_eq!(conv["messages"].as_array().unwrap().len(), 2);
    }
}
