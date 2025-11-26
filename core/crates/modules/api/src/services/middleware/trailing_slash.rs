use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

pub async fn remove_trailing_slash(request: Request, next: Next) -> Response {
    let uri = request.uri();
    let path = uri.path();

    if should_redirect(path) {
        let new_path = path.trim_end_matches('/');

        let new_uri = if let Some(query) = uri.query() {
            format!("{}?{}", new_path, query)
        } else {
            new_path.to_string()
        };

        return Redirect::permanent(&new_uri).into_response();
    }

    next.run(request).await
}

fn should_redirect(path: &str) -> bool {
    if path.len() <= 1 {
        return false;
    }

    if !path.ends_with('/') {
        return false;
    }

    if !path.starts_with("/api/") {
        return false;
    }

    if path.starts_with("/.well-known/") {
        return false;
    }

    if path.ends_with(".js/")
        || path.ends_with(".css/")
        || path.ends_with(".map/")
        || path.ends_with(".png/")
        || path.ends_with(".jpg/")
        || path.ends_with(".svg/")
        || path.ends_with(".ico/")
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_redirect_api_paths() {
        assert!(should_redirect("/api/v1/"));
        assert!(should_redirect("/api/v1/health/"));
        assert!(should_redirect("/api/v1/core/contexts/"));
    }

    #[test]
    fn test_should_not_redirect_root() {
        assert!(!should_redirect("/"));
    }

    #[test]
    fn test_should_not_redirect_without_trailing_slash() {
        assert!(!should_redirect("/api/v1"));
        assert!(!should_redirect("/api/v1/health"));
    }

    #[test]
    fn test_should_not_redirect_non_api_paths() {
        assert!(!should_redirect("/static/file.js/"));
        assert!(!should_redirect("/assets/image.png/"));
    }

    #[test]
    fn test_should_not_redirect_wellknown() {
        assert!(!should_redirect("/.well-known/openid-configuration/"));
    }

    #[test]
    fn test_should_not_redirect_static_assets() {
        assert!(!should_redirect("/app.js/"));
        assert!(!should_redirect("/style.css/"));
        assert!(!should_redirect("/logo.png/"));
    }
}
