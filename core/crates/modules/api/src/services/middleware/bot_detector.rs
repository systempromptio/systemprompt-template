use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct BotMarker {
    pub is_bot: bool,
    pub bot_type: BotType,
    pub user_agent: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
