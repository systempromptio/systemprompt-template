use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use systemprompt_core_database::{Database, DatabaseProvider};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Local,
    Docker,
    Production,
}

impl Environment {
    pub fn from_env() -> Self {
        match std::env::var("TEST_ENV")
            .unwrap_or_else(|_| "local".to_string())
            .as_str()
        {
            "docker" => Environment::Docker,
            "production" => Environment::Production,
            _ => Environment::Local,
        }
    }

    pub fn db_url(&self) -> String {
        match self {
            Environment::Local => std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://systemprompt:systemprompt_dev_password@localhost:5432/\
                 systemprompt_dev"
                    .to_string()
            }),
            Environment::Docker => std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://systemprompt:systemprompt_dev_password@postgres:5432/systemprompt_dev"
                    .to_string()
            }),
            Environment::Production => {
                std::env::var("DATABASE_URL").expect("DATABASE_URL required for production tests")
            },
        }
    }

    pub fn api_url(&self) -> String {
        match self {
            Environment::Local => std::env::var("API_EXTERNAL_URL")
                .or_else(|_| std::env::var("VITE_API_BASE_HOST"))
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            Environment::Docker => std::env::var("API_EXTERNAL_URL")
                .or_else(|_| std::env::var("VITE_API_BASE_HOST"))
                .unwrap_or_else(|_| "http://localhost:8085".to_string()),
            Environment::Production => std::env::var("API_EXTERNAL_URL")
                .or_else(|_| std::env::var("VITE_API_BASE_HOST"))
                .expect("API_EXTERNAL_URL required for production tests"),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Docker => "docker",
            Environment::Production => "production",
        }
    }
}

pub struct TestContext {
    pub db: Arc<Database>,
    pub http: Client,
    pub base_url: String,
    pub environment: Environment,
    fingerprint: String,
    admin_token: Option<String>,
    cached_token: Arc<Mutex<Option<String>>>,
}

impl TestContext {
    pub async fn new() -> Result<Self> {
        Self::with_environment(Environment::from_env()).await
    }

    pub async fn with_environment(env: Environment) -> Result<Self> {
        dotenvy::dotenv().ok();

        let db_url = env.db_url();
        let api_url = env.api_url();

        println!("📋 Test Environment: {}", env.name());
        println!("🔗 API URL: {}", api_url);
        println!("💾 Database: {}", mask_password(&db_url));

        let http = Client::builder()
            .timeout(Duration::from_secs(120))
            .user_agent("Mozilla/5.0 (Testing)")
            .cookie_store(true)
            .build()?;

        let db = Database::new_postgres(&db_url).await?;

        let admin_token = std::env::var("ADMIN_TOKEN").ok();

        Ok(Self {
            db: Arc::new(db),
            http,
            base_url: api_url,
            environment: env,
            fingerprint: Uuid::new_v4().to_string(),
            admin_token,
            cached_token: Arc::new(Mutex::new(None)),
        })
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub fn get_admin_token(&self) -> Option<String> {
        self.admin_token.clone()
    }

    pub async fn make_request(&self, path: &str) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .header("x-fingerprint", &self.fingerprint)
            .header("accept", "text/html,application/json")
            .header("accept-language", "en-US,en;q=0.9")
            .send()
            .await?;
        Ok(response)
    }

    pub async fn make_request_with_ua(
        &self,
        path: &str,
        user_agent: &str,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .header("user-agent", user_agent)
            // Don't override fingerprint - let middleware compute from user-agent + locale
            .header("accept", "text/html,application/json")
            .header("accept-language", "en-US,en;q=0.9")
            .send()
            .await?;
        Ok(response)
    }

    pub async fn cleanup(&self) -> Result<()> {
        use systemprompt_core_database::DatabaseQueryEnum;

        let find_query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(self.db.as_ref());
        let rows = self.db.fetch_all(&find_query, &[&self.fingerprint]).await?;

        for row in rows {
            if let Some(session_id) = row.get("session_id").and_then(|v| v.as_str()) {
                let delete_query = DatabaseQueryEnum::DeleteSessionById.get(self.db.as_ref());
                self.db.execute(&delete_query, &[&session_id]).await?;
            }
        }
        Ok(())
    }

    pub async fn get_anonymous_token(&self) -> Result<String> {
        {
            let cached = self.cached_token.lock().unwrap();
            if let Some(token) = cached.as_ref() {
                return Ok(token.clone());
            }
        }

        let url = format!("{}/api/v1/core/oauth/session", self.base_url);

        let mut retries = 0;
        let max_retries = 5;

        loop {
            let response = self
                .http
                .post(&url)
                .header("content-type", "application/json")
                .header("x-fingerprint", &self.fingerprint)
                .json(&serde_json::json!({
                    "client_id": "sp_web",
                    "metadata": {
                        "test": true,
                    }
                }))
                .send()
                .await?;

            let status = response.status();

            if status == 429 && retries < max_retries {
                retries += 1;
                let wait_ms = 100 * (1 << retries);
                tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                continue;
            }

            if !status.is_success() {
                let error_text = response.text().await?;
                return Err(anyhow::anyhow!(
                    "Failed to get anonymous token: {} - {}",
                    status,
                    error_text
                ));
            }

            let body: serde_json::Value = response.json().await?;
            let token = body["access_token"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("No access_token in response"))?
                .to_string();

            {
                let mut cached = self.cached_token.lock().unwrap();
                *cached = Some(token.clone());
            }

            return Ok(token);
        }
    }

    pub async fn create_context(&self, token: &str, name: &str) -> Result<String> {
        let url = format!("{}/api/v1/core/contexts", self.base_url);
        let response = self
            .http
            .post(&url)
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .header("x-fingerprint", &self.fingerprint)
            .json(&serde_json::json!({
                "name": name
            }))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Failed to create context: {} - {}",
                status,
                error_text
            ));
        }

        let body: serde_json::Value = response.json().await?;
        let context_id = body["context_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No context id in response"))?
            .to_string();

        Ok(context_id)
    }

    pub async fn make_authenticated_request(
        &self,
        method: reqwest::Method,
        path: &str,
        token: &str,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .request(method, &url)
            .header("authorization", format!("Bearer {}", token))
            .header("x-fingerprint", &self.fingerprint)
            .send()
            .await?;
        Ok(response)
    }

    pub async fn make_authenticated_json_request(
        &self,
        method: reqwest::Method,
        path: &str,
        token: &str,
        body: serde_json::Value,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .request(method, &url)
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .header("x-fingerprint", &self.fingerprint)
            .json(&body)
            .send()
            .await?;
        Ok(response)
    }
}

pub async fn wait_for_async_processing() {
    tokio::time::sleep(Duration::from_millis(5000)).await;
}

pub fn create_a2a_message(
    text: &str,
    context_id: &str,
) -> (String, String, String, serde_json::Value) {
    let task_id = format!("test-task-{}", Uuid::new_v4());
    let message_id = format!("msg-{}", Uuid::new_v4());

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "message/send",
        "params": {
            "message": {
                "messageId": &message_id,
                "contextId": context_id,
                "taskId": &task_id,
                "role": "user",
                "kind": "message",
                "parts": [{
                    "kind": "text",
                    "text": text
                }]
            },
            "configuration": null,
            "metadata": null
        },
        "id": &message_id
    });

    (task_id, context_id.to_string(), message_id, payload)
}

fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let before = &url[..colon_pos + 1];
            let after = &url[at_pos..];
            return format!("{}***{}", before, after);
        }
    }
    url.to_string()
}

#[derive(Debug, Clone)]
pub struct SessionData {
    pub session_id: String,
    pub user_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub request_count: i32,
    pub user_type: String,
    pub fingerprint_hash: Option<String>,
    pub landing_page: Option<String>,
    pub entry_url: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub referrer_url: Option<String>,
    pub referrer_source: Option<String>,
}

pub fn get_session_from_row(row: &HashMap<String, Value>) -> Result<SessionData> {
    let started_at = row
        .get("started_at")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    Ok(SessionData {
        session_id: row
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        user_id: row
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(String::from),
        started_at,
        request_count: row
            .get("request_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32,
        user_type: row
            .get("user_type")
            .and_then(|v| v.as_str())
            .unwrap_or("anon")
            .to_string(),
        fingerprint_hash: row
            .get("fingerprint_hash")
            .and_then(|v| v.as_str())
            .map(String::from),
        landing_page: row
            .get("landing_page")
            .and_then(|v| v.as_str())
            .map(String::from),
        entry_url: row
            .get("entry_url")
            .and_then(|v| v.as_str())
            .map(String::from),
        utm_source: row
            .get("utm_source")
            .and_then(|v| v.as_str())
            .map(String::from),
        utm_medium: row
            .get("utm_medium")
            .and_then(|v| v.as_str())
            .map(String::from),
        utm_campaign: row
            .get("utm_campaign")
            .and_then(|v| v.as_str())
            .map(String::from),
        referrer_url: row
            .get("referrer_url")
            .and_then(|v| v.as_str())
            .map(String::from),
        referrer_source: row
            .get("referrer_source")
            .and_then(|v| v.as_str())
            .map(String::from),
    })
}
