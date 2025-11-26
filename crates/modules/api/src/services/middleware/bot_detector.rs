use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct BotMarker {
    pub is_bot: bool,
    pub bot_type: BotType,
    pub user_agent: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BotType {
    KnownBot,
    Scanner,
    Suspicious,
    Human,
}

pub async fn detect_bots_early(mut req: Request, next: Next) -> Response {
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();

    let uri_path = req.uri().path().to_string();

    let marker = if is_known_bot(&user_agent) {
        BotMarker {
            is_bot: true,
            bot_type: BotType::KnownBot,
            user_agent: user_agent.clone(),
        }
    } else if is_scanner_request(&uri_path, &user_agent) {
        BotMarker {
            is_bot: false,
            bot_type: BotType::Scanner,
            user_agent: user_agent.clone(),
        }
    } else {
        BotMarker {
            is_bot: false,
            bot_type: BotType::Human,
            user_agent: user_agent.clone(),
        }
    };

    req.extensions_mut().insert(Arc::new(marker));
    next.run(req).await
}

fn is_known_bot(user_agent: &str) -> bool {
    let bot_patterns = [
        "Googlebot",
        "bingbot",
        "Slurp",
        "DuckDuckBot",
        "Baiduspider",
        "YandexBot",
        "facebookexternalhit",
        "Twitterbot",
        "LinkedInBot",
        "WhatsApp",
        "TelegramBot",
        "Discordbot",
        "ia_archiver",
        "curl",
        "wget",
        "python",
        "java",
        "perl",
        "ruby",
        "go-http-client",
        "Node",
        "scrapy",
        "urllib",
        "requests",
        "okhttp",
        "httpclient",
    ];

    let ua_lower = user_agent.to_lowercase();
    bot_patterns
        .iter()
        .any(|pattern| ua_lower.contains(&pattern.to_lowercase()))
}

fn is_scanner_request(path: &str, user_agent: &str) -> bool {
    let scanner_paths = [
        ".env",
        ".git",
        ".php",
        "admin",
        "wp-admin",
        "wp-login",
        "administrator",
        ".sql",
        ".backup",
        "config.php",
        "web.config",
        ".well-known",
    ];

    let scanner_agents = [
        "masscan",
        "nmap",
        "nikto",
        "sqlmap",
        "metasploit",
        "nessus",
        "openvas",
        "zap",
        "burp",
        "qualys",
    ];

    let path_lower = path.to_lowercase();
    let ua_lower = user_agent.to_lowercase();

    scanner_paths.iter().any(|p| path_lower.contains(p))
        || scanner_agents.iter().any(|a| ua_lower.contains(a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_bot_detection() {
        assert!(is_known_bot(
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"
        ));
        assert!(is_known_bot("Mozilla/5.0 (compatible; bingbot/2.0)"));
        assert!(is_known_bot("facebookexternalhit/1.1"));
    }

    #[test]
    fn test_human_detection() {
        assert!(!is_known_bot(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
        ));
        assert!(!is_known_bot(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
        ));
    }

    #[test]
    fn test_scanner_path_detection() {
        assert!(is_scanner_request("/.env", "Mozilla/5.0"));
        assert!(is_scanner_request("/admin", "Mozilla/5.0"));
        assert!(is_scanner_request("/wp-admin", "Mozilla/5.0"));
        assert!(is_scanner_request("/.git", "Mozilla/5.0"));
    }

    #[test]
    fn test_scanner_agent_detection() {
        assert!(is_scanner_request("/", "masscan"));
        assert!(is_scanner_request("/", "nmap"));
        assert!(is_scanner_request("/", "sqlmap/1.0"));
    }
}
