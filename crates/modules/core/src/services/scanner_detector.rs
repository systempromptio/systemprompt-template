/// Detects malicious scanner traffic patterns
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
            // Admin/management paths
            || path.contains("/admin")
            || path.contains("/wp-admin")
            || path.contains("/wp-content")
            || path.contains("/uploads")
            || path.contains("/cgi-bin")
            || path.contains("/phpmyadmin")
            || path.contains("/xmlrpc")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_paths() {
        assert!(ScannerDetector::is_scanner_path("/.env"));
        assert!(ScannerDetector::is_scanner_path("/test.php"));
        assert!(ScannerDetector::is_scanner_path("/admin.php"));
        assert!(ScannerDetector::is_scanner_path("/wp-admin/admin.php"));
        assert!(!ScannerDetector::is_scanner_path("/blog/post"));
        assert!(!ScannerDetector::is_scanner_path("/api/v1/users"));
    }

    #[test]
    fn test_scanner_agents() {
        // Original scanner detection
        assert!(ScannerDetector::is_scanner_agent("masscan/1.0"));
        assert!(ScannerDetector::is_scanner_agent("curl/7.68.0"));

        // New scanner detection - reconnaissance tools
        assert!(ScannerDetector::is_scanner_agent("Mozilla/5.0 zgrab/0.x"));
        assert!(ScannerDetector::is_scanner_agent(
            "Hello from Palo Alto Networks"
        ));

        // New scanner detection - automated probing
        assert!(ScannerDetector::is_scanner_agent(
            "probe-image-size/7.2.3(+https://github.com/nodeca/probe-image-size)"
        ));
        assert!(ScannerDetector::is_scanner_agent("libredtail-http"));

        // New scanner detection - WordPress bots
        assert!(ScannerDetector::is_scanner_agent(
            "WordPress/6.8.3; https://ai.jiayun.info"
        ));

        // New scanner detection - empty/short user agents
        assert!(ScannerDetector::is_scanner_agent(""));
        assert!(ScannerDetector::is_scanner_agent("short"));

        // New scanner detection - generic Mozilla
        assert!(ScannerDetector::is_scanner_agent("Mozilla/5.0"));

        // New scanner detection - old browsers
        assert!(ScannerDetector::is_scanner_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.113 Safari/537.36"));
        assert!(ScannerDetector::is_scanner_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.108 Safari/537.36"));
        assert!(ScannerDetector::is_scanner_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_7_0) AppleWebKit/535.11 (KHTML, like Gecko) Chrome/17.0.963.56 Safari/535.11"));
        assert!(ScannerDetector::is_scanner_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:132.0) Gecko/20100101 Firefox/80.0"
        ));

        // Should NOT flag modern browsers
        assert!(!ScannerDetector::is_scanner_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36"));
        assert!(!ScannerDetector::is_scanner_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148"));
    }

    #[test]
    fn test_null_user_agent() {
        // NULL user agent should be flagged as scanner
        assert!(ScannerDetector::is_scanner(Some("/"), None, None, None));
    }

    #[test]
    fn test_high_velocity() {
        assert!(ScannerDetector::is_high_velocity(100, 60)); // 100 requests in 60s = 100 req/min
        assert!(ScannerDetector::is_high_velocity(50, 30)); // 50 requests in 30s = 100 req/min
        assert!(!ScannerDetector::is_high_velocity(10, 60)); // 10 requests in 60s = 10 req/min
    }
}
