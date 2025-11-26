use anyhow::Result;
use systemprompt_core_mcp::services::orchestrator::McpEvent;

#[tokio::test]
async fn test_event_bus_publishes_events() -> Result<()> {
    use systemprompt_core_mcp::services::orchestrator::EventBus;

    let event_bus = EventBus::new(10);
    let mut receiver = event_bus.subscribe();

    event_bus
        .publish(McpEvent::ServiceStartRequested {
            service_name: "test-service".to_string(),
        })
        .await?;

    let received = receiver.recv().await?;
    assert_eq!(received.event_type(), "service_start_requested");
    assert_eq!(received.service_name(), "test-service");

    Ok(())
}

#[tokio::test]
async fn test_event_types() -> Result<()> {
    let events = vec![
        (
            McpEvent::ServiceStartRequested {
                service_name: "test".to_string(),
            },
            "service_start_requested",
        ),
        (
            McpEvent::ServiceStarted {
                service_name: "test".to_string(),
                process_id: 123,
                port: 8080,
            },
            "service_started",
        ),
        (
            McpEvent::ServiceFailed {
                service_name: "test".to_string(),
                error: "error".to_string(),
            },
            "service_failed",
        ),
        (
            McpEvent::ServiceStopped {
                service_name: "test".to_string(),
                exit_code: Some(0),
            },
            "service_stopped",
        ),
        (
            McpEvent::HealthCheckFailed {
                service_name: "test".to_string(),
                reason: "timeout".to_string(),
            },
            "health_check_failed",
        ),
        (
            McpEvent::SchemaUpdated {
                service_name: "test".to_string(),
                tool_count: 5,
            },
            "schema_updated",
        ),
        (
            McpEvent::ServiceRestartRequested {
                service_name: "test".to_string(),
                reason: "health check".to_string(),
            },
            "service_restart_requested",
        ),
    ];

    for (event, expected_type) in events {
        assert_eq!(event.event_type(), expected_type);
        assert_eq!(event.service_name(), "test");
    }

    Ok(())
}

#[tokio::test]
async fn test_event_serialization() -> Result<()> {
    let event = McpEvent::ServiceStarted {
        service_name: "test-service".to_string(),
        process_id: 12345,
        port: 8080,
    };

    let json = serde_json::to_string(&event)?;
    let deserialized: McpEvent = serde_json::from_str(&json)?;

    assert_eq!(deserialized.event_type(), "service_started");
    assert_eq!(deserialized.service_name(), "test-service");

    Ok(())
}
