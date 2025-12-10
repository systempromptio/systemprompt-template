/// Detects malicious scanner traffic patterns
#[derive(Debug, Clone, Copy)]
pub struct ScannerDetector;

impl ScannerDetector {
    /// Check if path matches known scanner patterns
    /// Case-sensitivity is intentional here - attackers use exact case
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    pub fn is_scanner_path(path: &str) -> bool {
        // File extensions commonly scanned by attackers
        path.ends_with(".php")
            || path.ends_with(".env")
            || path.ends_with(".git")
            || path.ends_with(".sql")
            || path.ends_with(".bak")
            || path.ends_with(".old")
            || path.ends_with(".zip")
            || path.ends_with(".tar.gz")
            || path.ends_with(".db")
            || path.ends_with(".config")
            || path.ends_with(".cgi")
            || path.ends_with(".htm")
            // Admin/management paths
            || path.contains("/admin")
            || path.contains("/wp-admin")
            || path.contains("/wp-content")
            || path.contains("/uploads")
            || path.contains("/cgi-bin")
            || path.contains("/phpmyadmin")
            || path.contains("/xmlrpc")
            // Router/IoT exploit paths
            || path.contains("/luci")
            || path.contains("/ssi.cgi")
            || path.contains("internal_forms_authentication")
            || path.contains("/identity")
            || path.contains("/Login.htm")
            || path.contains("/manager/html")
            || path.contains("/config/")
            || path.contains("/setup.cgi")
            // Common exploit paths
            || path.contains("/eval-stdin.php")
            || path.contains("/shell.php")
            || path.contains("/c99.php")
    }

    /// Check if user agent indicates scanner/tool
    pub fn is_scanner_agent(user_agent: &str) -> bool {
        let ua_lower = user_agent.to_lowercase();

        // Empty or suspiciously short user agents
        if user_agent.is_empty() || user_agent.len() < 10 {
            return true;
        }

        // Generic "Mozilla/5.0" with no details
        if user_agent == "Mozilla/5.0" || user_agent.trim() == "Mozilla/5.0" {
            return true;
        }

        // Security scanners and penetration testing tools
        ua_lower.contains("masscan")
            || ua_lower.contains("nmap")
            || ua_lower.contains("nikto")
            || ua_lower.contains("sqlmap")
            || ua_lower.contains("havij")
            || ua_lower.contains("acunetix")
            || ua_lower.contains("nessus")
            || ua_lower.contains("openvas")
            || ua_lower.contains("w3af")
            || ua_lower.contains("metasploit")
            || ua_lower.contains("burpsuite")
            || ua_lower.contains("zap")
            // Network reconnaissance tools
            || ua_lower.contains("zgrab")
            || ua_lower.contains("censys")
            || ua_lower.contains("shodan")
            || ua_lower.contains("masscan")
            // Security vendors
            || ua_lower.contains("palo alto")
            || ua_lower.contains("cortex")
            || ua_lower.contains("xpanse")
            // Automated probing tools
            || ua_lower.contains("probe-image-size")
            || ua_lower.contains("libredtail")
            || ua_lower.contains("httpclient")
            || ua_lower.contains("httpunit")
            || ua_lower.contains("java/")
            // WordPress bots and pingbacks
            || ua_lower.starts_with("wordpress/")
            || ua_lower.contains("wp-http")
            || ua_lower.contains("wp-cron")
            // Generic HTTP clients
            || (ua_lower.contains("curl") && ua_lower.len() < 20)
            || (ua_lower.contains("wget") && ua_lower.len() < 20)
            || (ua_lower.contains("python-requests") && ua_lower.len() < 30)
            || (ua_lower.contains("go-http-client") && ua_lower.len() < 30)
            || (ua_lower.contains("ruby") && ua_lower.len() < 25)
            // Old browser versions (likely spoofed/automated)
            || Self::is_outdated_browser(&ua_lower)
    }

    /// Detect outdated browsers (likely fake/automated traffic)
    fn is_outdated_browser(ua_lower: &str) -> bool {
        // Chrome versions older than 3 years (before v90, released 2021)
        if ua_lower.contains("chrome/") {
            if let Some(pos) = ua_lower.find("chrome/") {
                let version_str = &ua_lower[pos + 7..];
                if let Some(dot_pos) = version_str.find('.') {
                    if let Ok(major) = version_str[..dot_pos].parse::<i32>() {
                        if major < 90 {
                            return true;
                        }
                    }
                }
            }
        }

        // Firefox versions older than 3 years (before v88, released 2021)
        if ua_lower.contains("firefox/") {
            if let Some(pos) = ua_lower.find("firefox/") {
                let version_str = &ua_lower[pos + 8..];
                if let Some(space_pos) = version_str.find(|c: char| !c.is_numeric() && c != '.') {
                    if let Ok(major) = version_str[..space_pos].parse::<i32>() {
                        if major < 88 {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Detect high-velocity scanning behavior
    pub fn is_high_velocity(request_count: i64, duration_seconds: i64) -> bool {
        if duration_seconds < 1 {
            return false;
        }

        let requests_per_minute = (request_count as f64 / duration_seconds as f64) * 60.0;

        // More than 30 requests per minute is likely automated
        requests_per_minute > 30.0
    }

    /// Comprehensive scanner detection
    pub fn is_scanner(
        path: Option<&str>,
        user_agent: Option<&str>,
        request_count: Option<i64>,
        duration_seconds: Option<i64>,
    ) -> bool {
        // Check path patterns
        if let Some(p) = path {
            if Self::is_scanner_path(p) {
                return true;
            }
        }

        // Check user agent
        match user_agent {
            Some(ua) => {
                if Self::is_scanner_agent(ua) {
                    return true;
                }
            },
            None => {
                // NULL user agent is highly suspicious
                return true;
            },
        }

        // Check velocity
        if let (Some(count), Some(duration)) = (request_count, duration_seconds) {
            if Self::is_high_velocity(count, duration) {
                return true;
            }
        }

        false
    }
}
