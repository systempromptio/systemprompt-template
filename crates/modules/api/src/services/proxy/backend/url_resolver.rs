#[derive(Debug, Clone, Copy)]
pub struct UrlResolver;

impl UrlResolver {
    pub fn build_backend_url(protocol: &str, host: &str, port: i32, path: &str) -> String {
        let clean_path = path.trim_start_matches('/');

        match protocol {
            "mcp" => {
                if clean_path.is_empty() || clean_path == "mcp" {
                    format!("http://{host}:{port}/mcp")
                } else {
                    format!("http://{host}:{port}/{clean_path}")
                }
            },
            _ => {
                if clean_path.is_empty() {
                    format!("http://{host}:{port}/")
                } else {
                    format!("http://{host}:{port}/{clean_path}")
                }
            },
        }
    }

    pub fn append_query_params(url: String, query: Option<&str>) -> String {
        match query {
            Some(q) if !q.is_empty() => format!("{url}?{q}"),
            _ => url,
        }
    }
}
