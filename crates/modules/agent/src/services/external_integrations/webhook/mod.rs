//! Webhook management for external event handling.

mod service;

pub use service::{
    RetryPolicy, WebhookConfig, WebhookDeliveryResult, WebhookService, WebhookTestResult,
};
