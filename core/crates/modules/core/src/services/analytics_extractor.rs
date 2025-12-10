use axum::extract::Request;
use axum::http::{HeaderMap, Uri};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SessionAnalytics {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub fingerprint_hash: Option<String>,
    pub preferred_locale: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub referrer_source: Option<String>,
    pub referrer_url: Option<String>,
    pub landing_page: Option<String>,
    pub entry_url: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
}

pub type GeoIpReader = Arc<maxminddb::Reader<Vec<u8>>>;

impl SessionAnalytics {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        Self::from_headers_with_geoip(headers, None)
    }

    pub fn from_headers_with_geoip(
        headers: &HeaderMap,
        geoip_reader: Option<&GeoIpReader>,
    ) -> Self {
        Self::from_headers_with_geoip_and_socket(headers, geoip_reader, None)
    }

    pub fn from_headers_with_geoip_and_socket(
        headers: &HeaderMap,
        geoip_reader: Option<&GeoIpReader>,
        socket_addr: Option<std::net::SocketAddr>,
    ) -> Self {
        let user_agent = headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);

        let ip_address = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
            .or_else(|| {
                headers
                    .get("x-real-ip")
                    .and_then(|v| v.to_str().ok())
                    .map(ToString::to_string)
            })
            .or_else(|| socket_addr.map(|addr| addr.ip().to_string()));

        let fingerprint_hash = headers
            .get("x-fingerprint")
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);

        let preferred_locale = headers
            .get("accept-language")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().split(';').next().unwrap_or(s).to_string());

        let (device_type, browser, os) = user_agent
            .as_ref()
            .map_or((None, None, None), |ua| Self::parse_user_agent(ua));

        let (country, region, city) = ip_address
            .as_ref()
            .and_then(|ip_str| Self::lookup_geoip(ip_str, geoip_reader))
            .unwrap_or((None, None, None));

        let referrer_url = headers
            .get("referer")
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);

        let referrer_source = referrer_url
            .as_ref()
            .and_then(|url| Self::parse_referrer_source(url));

        Self {
            ip_address,
            user_agent,
            device_type,
            browser,
            os,
            fingerprint_hash,
            preferred_locale,
            country,
            region,
            city,
            referrer_source,
            referrer_url,
            landing_page: None,
            entry_url: None,
            utm_source: None,
            utm_medium: None,
            utm_campaign: None,
        }
    }

    pub fn from_headers_and_uri(
        headers: &HeaderMap,
        uri: Option<&Uri>,
        geoip_reader: Option<&GeoIpReader>,
        content_config: Option<&systemprompt_models::ContentConfig>,
    ) -> Self {
        let mut analytics = Self::from_headers_with_geoip(headers, geoip_reader);

        if let Some(uri) = uri {
            let query_params = Self::parse_query_params(uri);

            analytics.utm_source = query_params.get("utm_source").cloned();
            analytics.utm_medium = query_params.get("utm_medium").cloned();
            analytics.utm_campaign = query_params.get("utm_campaign").cloned();

            let is_html_page = content_config.is_some_and(|cfg| cfg.is_html_page(uri.path()));

            if is_html_page {
                analytics.entry_url = Some(uri.to_string());
                if analytics.referrer_url.is_none()
                    || Self::is_same_site(analytics.referrer_url.as_ref())
                {
                    analytics.landing_page = Some(uri.path().to_string());
                }
            }
        }

        analytics
    }

    pub fn from_request(
        request: &Request,
        geoip_reader: Option<&GeoIpReader>,
        content_config: Option<&systemprompt_models::ContentConfig>,
    ) -> Self {
        Self::from_headers_and_uri(
            request.headers(),
            Some(request.uri()),
            geoip_reader,
            content_config,
        )
    }

    fn parse_query_params(uri: &Uri) -> HashMap<String, String> {
        uri.query()
            .map(|q| {
                q.split('&')
                    .filter_map(|param| {
                        let mut parts = param.splitn(2, '=');
                        Some((parts.next()?.to_string(), parts.next()?.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn is_same_site(referrer_url: Option<&String>) -> bool {
        referrer_url
            .and_then(|url| url::Url::parse(url).ok())
            .and_then(|parsed| parsed.host_str().map(ToString::to_string))
            .is_some_and(|host| {
                host.contains("systemprompt.io")
                    || host == "localhost"
                    || host.contains("tyingshoelaces.com")
            })
    }

    fn lookup_geoip(
        ip_str: &str,
        geoip_reader: Option<&GeoIpReader>,
    ) -> Option<(Option<String>, Option<String>, Option<String>)> {
        let reader = geoip_reader?;
        let ip: std::net::IpAddr = ip_str.parse().ok()?;

        let city_data: maxminddb::geoip2::City = reader.lookup(ip).ok()?;

        let country = city_data
            .country
            .and_then(|c| c.iso_code)
            .map(ToString::to_string);

        let region = city_data
            .subdivisions
            .and_then(|subdivisions| subdivisions.first().cloned())
            .and_then(|s| s.iso_code)
            .map(ToString::to_string);

        let city_name = city_data
            .city
            .and_then(|c| c.names)
            .and_then(|names| names.get("en").copied())
            .map(ToString::to_string);

        Some((country, region, city_name))
    }

    fn parse_user_agent(ua: &str) -> (Option<String>, Option<String>, Option<String>) {
        let ua_lower = ua.to_lowercase();

        let device_type = if ua_lower.contains("mobile")
            || ua_lower.contains("android")
            || ua_lower.contains("iphone")
        {
            Some("mobile".to_string())
        } else if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
            Some("tablet".to_string())
        } else {
            Some("desktop".to_string())
        };

        let browser = if ua_lower.contains("edg/") || ua_lower.contains("edge") {
            Some("Edge".to_string())
        } else if ua_lower.contains("chrome") && !ua_lower.contains("edg") {
            Some("Chrome".to_string())
        } else if ua_lower.contains("firefox") {
            Some("Firefox".to_string())
        } else if ua_lower.contains("safari") && !ua_lower.contains("chrome") {
            Some("Safari".to_string())
        } else if ua_lower.contains("opera") || ua_lower.contains("opr/") {
            Some("Opera".to_string())
        } else {
            None
        };

        let os = if ua_lower.contains("windows") {
            Some("Windows".to_string())
        } else if ua_lower.contains("mac os x") || ua_lower.contains("macos") {
            Some("macOS".to_string())
        } else if ua_lower.contains("linux") {
            Some("Linux".to_string())
        } else if ua_lower.contains("android") {
            Some("Android".to_string())
        } else if ua_lower.contains("iphone")
            || ua_lower.contains("ipad")
            || ua_lower.contains("ios")
        {
            Some("iOS".to_string())
        } else {
            None
        };

        (device_type, browser, os)
    }

    fn parse_referrer_source(url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()
            .and_then(|parsed_url| parsed_url.host_str().map(ToString::to_string))
            .and_then(|host| {
                if host.parse::<std::net::IpAddr>().is_ok() {
                    None
                } else {
                    Some(host)
                }
            })
    }

    pub fn is_bot(&self) -> bool {
        if let Some(ua) = &self.user_agent {
            Self::matches_bot_pattern(ua)
        } else {
            false
        }
    }

    fn matches_bot_pattern(user_agent: &str) -> bool {
        const BOT_KEYWORDS: &[&str] = &[
            "bot",
            "crawler",
            "spider",
            "scraper",
            "crawling",
            "googlebot",
            "google-inspectiontool",
            "adsbot-google",
            "googleother",
            "bingbot",
            "bingpreview",
            "msnbot",
            "baiduspider",
            "yandexbot",
            "yandex.com/bots",
            "duckduckbot",
            "slurp",
            "yahoo",
            "facebookexternalhit",
            "facebookcatalog",
            "facebot",
            "meta-externalagent",
            "twitterbot",
            "linkedinbot",
            "slackbot",
            "discordbot",
            "whatsapp",
            "telegrambot",
            "pinterestbot",
            "chatgpt-user",
            "gptbot",
            "claude-web",
            "anthropic-ai",
            "perplexitybot",
            "cohere-ai",
            "petalbot",
            "bytespider",
            "sogou",
            "amazonbot",
            "applebot",
            "dotbot",
            "semrushbot",
            "ahrefsbot",
            "majesticbot",
            "mj12bot",
            "rogerbot",
            "exabot",
            "sistrix",
            "seolyt",
            "barkrowler",
            "blexbot",
            "bubing",
            "cliqzbot",
            "uptimerobot",
            "pingdom",
            "statuscake",
            "site24x7",
            "lighthouse",
            "pagespeed",
            "speedcurve",
            "headless",
            "phantom",
            "selenium",
            "webdriver",
            "puppeteer",
            "archive.org_bot",
            "ia_archiver",
            "embedly",
            "flipboard",
            "google-structured-data-testing-tool",
            "scrapy",
            "python-requests",
            "python-urllib",
            "curl",
            "wget",
            "libwww",
            "http.rb",
            "guzzlehttp",
            "okhttp",
            "apache-httpclient",
            "go-http-client",
            "node-fetch",
            "axios",
        ];

        let ua_lower = user_agent.to_lowercase();

        if BOT_KEYWORDS
            .iter()
            .any(|keyword| ua_lower.contains(keyword))
        {
            return true;
        }

        if user_agent.len() < 10 {
            return true;
        }

        if ua_lower.contains("compatible")
            && !ua_lower.contains("chrome")
            && !ua_lower.contains("firefox")
            && !ua_lower.contains("safari")
            && !ua_lower.contains("edge")
        {
            return true;
        }

        false
    }

    pub fn is_bot_ip(&self) -> bool {
        if let Some(ip) = &self.ip_address {
            Self::matches_bot_ip_range(ip)
        } else {
            false
        }
    }

    fn matches_bot_ip_range(ip: &str) -> bool {
        const BOT_IP_PREFIXES: &[&str] = &[
            "66.249.", "40.77.", "157.55.", "207.46.", "69.171.", "173.252.", "31.13.",
        ];

        BOT_IP_PREFIXES.iter().any(|prefix| ip.starts_with(prefix))
    }
}
