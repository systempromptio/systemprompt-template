#[derive(Debug, Clone)]
pub struct ClientPool {
    default_client: reqwest::Client,
}

impl ClientPool {
    pub fn new() -> Self {
        Self {
            default_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    pub fn get_default_client(&self) -> reqwest::Client {
        self.default_client.clone()
    }
}
