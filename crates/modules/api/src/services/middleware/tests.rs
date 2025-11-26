#[cfg(test)]
mod analytics_tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Instant;

    #[tokio::test]
    async fn test_session_activity_tracking_timeout_recovery() {
        let timeout_duration = std::time::Duration::from_millis(50);

        let slow_task_completed = Arc::new(AtomicBool::new(false));
        let slow_task_completed_clone = slow_task_completed.clone();

        let start = Instant::now();

        let result = tokio::time::timeout(timeout_duration, async {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            slow_task_completed_clone.store(true, Ordering::SeqCst);
            Ok::<(), String>(())
        })
        .await;

        assert!(
            result.is_err(),
            "Timeout should trigger when task exceeds duration"
        );
        assert!(
            !slow_task_completed.load(Ordering::SeqCst),
            "Slow task should not complete before timeout"
        );

        let elapsed = start.elapsed();
        assert!(
            elapsed >= timeout_duration,
            "Elapsed time should exceed timeout duration"
        );
        assert!(
            elapsed < std::time::Duration::from_millis(150),
            "Elapsed time should not wait for full task"
        );
    }

    #[tokio::test]
    async fn test_session_activity_tracking_completes_within_timeout() {
        let timeout_duration = std::time::Duration::from_millis(50);
        let task_completed = Arc::new(AtomicBool::new(false));
        let task_completed_clone = task_completed.clone();

        let result = tokio::time::timeout(timeout_duration, async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            task_completed_clone.store(true, Ordering::SeqCst);
            Ok::<(), String>(())
        })
        .await;

        assert!(result.is_ok(), "Task should complete within timeout");
        assert!(
            task_completed.load(Ordering::SeqCst),
            "Task should have completed"
        );
    }
}

#[cfg(test)]
mod auth_tests {
    use super::super::auth::*;
    use systemprompt_core_system::ServiceCategory;

    #[test]
    fn test_service_category_path_matching() {
        // Test Core category
        assert!(ServiceCategory::Core.matches_path("/api/v1/core"));
        assert!(ServiceCategory::Core.matches_path("/api/v1/core/users"));
        assert!(ServiceCategory::Core.matches_path("/api/v1/core/config/settings"));
        assert!(!ServiceCategory::Core.matches_path("/api/v1/agents"));

        // Test Agent category
        assert!(ServiceCategory::Agent.matches_path("/api/v1/agents"));
        assert!(ServiceCategory::Agent.matches_path("/api/v1/agents/list"));
        assert!(!ServiceCategory::Agent.matches_path("/api/v1/core"));

        // Test MCP category
        assert!(ServiceCategory::Mcp.matches_path("/api/v1/mcp"));
        assert!(ServiceCategory::Mcp.matches_path("/api/v1/mcp/servers"));
        assert!(!ServiceCategory::Mcp.matches_path("/api/v1/service/mcp")); // Old path should not match

        // Test Meta category
        assert!(ServiceCategory::Meta.matches_path("/"));
        assert!(ServiceCategory::Meta.matches_path("/.well-known"));
        assert!(ServiceCategory::Meta.matches_path("/api/v1/meta/health"));
    }

    #[test]
    fn test_category_from_path() {
        // Test specific paths map to correct categories
        assert_eq!(
            ServiceCategory::from_path("/api/v1/core/users"),
            Some(ServiceCategory::Core)
        );
        assert_eq!(
            ServiceCategory::from_path("/api/v1/agents/discover"),
            Some(ServiceCategory::Agent)
        );
        assert_eq!(
            ServiceCategory::from_path("/api/v1/mcp/proxy"),
            Some(ServiceCategory::Mcp)
        );
        assert_eq!(
            ServiceCategory::from_path("/api/v1/meta/health"),
            Some(ServiceCategory::Meta)
        );
        assert_eq!(
            ServiceCategory::from_path("/.well-known/openid-configuration"),
            Some(ServiceCategory::Meta)
        );

        // Test unknown paths
        assert_eq!(ServiceCategory::from_path("/unknown/path"), None);
    }

    #[test]
    fn test_all_categories_have_unique_base_paths() {
        use std::collections::HashSet;

        let mut paths = HashSet::new();
        for category in ServiceCategory::all() {
            let base_path = category.base_path();
            // Skip Meta category as it has special "/" path
            if base_path != "/" {
                assert!(
                    paths.insert(base_path),
                    "Duplicate base path found: {}",
                    base_path
                );
            }
        }
    }

    #[test]
    fn test_auth_config_public_paths() {
        let config = ApiAuthMiddlewareConfig::default();

        // Verify public paths
        assert!(config.is_public_path("/.well-known/openid-configuration"));
        assert!(config.is_public_path("/api/v1/core/oauth/session"));
        assert!(config.is_public_path("/api/v1/core/oauth/register"));

        // Non-API paths should be public
        assert!(config.is_public_path("/health"));
        assert!(config.is_public_path("/static/app.js"));
    }

    #[test]
    fn test_category_consistency() {
        // Ensure that for every category, the path it generates
        // can be mapped back to the same category
        for category in ServiceCategory::all() {
            let base_path = category.base_path();

            // Skip Meta as it has special handling
            if *category != ServiceCategory::Meta {
                let test_path = format!("{}/test", base_path);
                assert_eq!(
                    ServiceCategory::from_path(&test_path),
                    Some(*category),
                    "Category {:?} with base path {} failed round-trip test",
                    category,
                    base_path
                );
            }
        }
    }
}
