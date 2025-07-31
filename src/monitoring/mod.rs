//! Monitoring and observability modules for Warp Terminal
//! 
//! This module provides comprehensive monitoring capabilities including:
//! - Error reporting via Sentry
//! - Performance monitoring and tracing
//! - Application health metrics
//! - Custom event tracking

pub mod sentry_integration;

// Re-export commonly used types and functions
pub use sentry_integration::{
    SentryConfig, SentryIntegration, init_sentry, sentry
};

use tracing_subscriber::prelude::*;

/// Initialize all monitoring systems
pub fn init_monitoring() -> anyhow::Result<()> {
    // Initialize Sentry with default configuration
    let sentry_config = SentryConfig::default();
    init_sentry(sentry_config)?;
    
    // Initialize tracing integration with Sentry
    if let Some(sentry_instance) = sentry() {
        if sentry_instance.is_enabled() {
            // Setup sentry-tracing integration
            let layer = sentry_tracing::layer();
            tracing_subscriber::registry()
                .with(layer)
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .init();
        }
    }
    
    Ok(())
}

/// Monitor application startup
pub fn monitor_startup() {
    if let Some(sentry) = sentry() {
        sentry.add_breadcrumb("Application startup initiated", "app.lifecycle", sentry::Level::Info);
        
        // Start startup transaction
        if let Some(_transaction) = sentry.start_transaction("app.startup", "app.lifecycle") {
            // Transaction will be automatically finished when dropped
        }
    }
}

/// Monitor application shutdown
pub fn monitor_shutdown() {
    if let Some(sentry) = sentry() {
        sentry.add_breadcrumb("Application shutdown initiated", "app.lifecycle", sentry::Level::Info);
        sentry.capture_message("Application shutting down gracefully", sentry::Level::Info);
    }
}

/// Report a critical error that requires immediate attention
pub fn report_critical_error(error: &anyhow::Error, context: &str) {
    if let Some(sentry) = sentry() {
        sentry.capture_error(error, Some(context));
    }
    // Also log to stderr for immediate visibility
    eprintln!("CRITICAL ERROR: {}: {}", context, error);
}

/// Track custom events for analytics
pub fn track_event(event_name: &str, properties: std::collections::HashMap<String, String>) {
    if let Some(sentry) = sentry() {
        sentry.add_breadcrumb(
            &format!("Event: {}", event_name),
            "analytics",
            sentry::Level::Info
        );
        
        // Add properties as context
        let context = sentry::protocol::Context::Other({
            let mut map = sentry::protocol::Map::new();
            for (key, value) in properties {
                map.insert(key, value.into());
            }
            map
        });
        
        sentry.set_context("event_properties", context);
    }
}
