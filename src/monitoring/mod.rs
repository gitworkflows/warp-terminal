//! Monitoring and observability modules for Warp Terminal
//! 
//! This module provides comprehensive monitoring capabilities including:
//! - Error reporting via Sentry
//! - Performance monitoring and tracing
//! - Application health metrics
//! - Custom event tracking

pub mod sentry_integration;

// Re-export commonly used types and functions
#[cfg(feature = "sentry")]
pub use sentry_integration::{
    SentryConfig, SentryIntegration, init_sentry, sentry
};

#[cfg(feature = "sentry")]
use anyhow::Context;
use tracing::{info, warn};
#[cfg(feature = "sentry")]
use tracing_subscriber::prelude::*;

#[cfg(feature = "sentry")]
use sentry_tracing;
/// Initialize all monitoring systems with environment-aware configuration
#[cfg(feature = "sentry")]
pub fn init_monitoring() -> anyhow::Result<()> {
    // Initialize Sentry with environment-based configuration
    let sentry_integration = SentryIntegration::with_environment_config()
        .context("Failed to initialize Sentry with environment configuration")?;
    
    // Set the global instance using the proper initialization function
    let config = sentry_integration.config().clone();
    init_sentry(config)?;
    
    // Initialize tracing integration with Sentry
    if let Some(sentry_instance) = sentry() {
        if sentry_instance.is_enabled() {
            info!("Initialized monitoring with DSN: {}", 
                sentry_instance.dsn().unwrap_or("<not configured>"));
                
            // Setup sentry-tracing integration
            let layer = sentry_tracing::layer();
            tracing_subscriber::registry()
                .with(layer)
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .init();
                
            // Log health status
            if sentry_instance.check_health() {
                info!("Sentry is healthy and ready to capture events");
            } else {
                warn!("Sentry is enabled but not active - events may not be captured");
            }
        } else {
            info!("Sentry is disabled - no DSN provided");
        }
    }
    
    Ok(())
}

/// Monitor application startup
#[cfg(feature = "sentry")]
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
#[cfg(feature = "sentry")]
pub fn monitor_shutdown() {
    if let Some(sentry) = sentry() {
        sentry.add_breadcrumb("Application shutdown initiated", "app.lifecycle", sentry::Level::Info);
        sentry.capture_message("Application shutting down gracefully", sentry::Level::Info);
    }
}

/// Report a critical error that requires immediate attention
#[cfg(feature = "sentry")]
pub fn report_critical_error(error: &anyhow::Error, context: &str) {
    report_error_with_tags(error, Some(context), None);
    
    // Also log to stderr for immediate visibility
    eprintln!("CRITICAL ERROR: {}: {}", context, error);
}

/// Report an error with additional tags
#[cfg(feature = "sentry")]
pub fn report_error_with_tags(
    error: &anyhow::Error,
    context: Option<&str>,
    tags: Option<std::collections::HashMap<String, String>>,
) {
    if let Some(sentry) = sentry() {
        if !sentry.check_health() {
            warn!("Attempted to report error but Sentry is not healthy - event may be lost");
        }
        sentry.capture_error_with_tags(error, context, tags);
    } else {
        tracing::error!("Sentry not initialized - cannot report error: {:?}", error);
    }
}

/// Time an operation and report it as a transaction
#[cfg(feature = "sentry")]
pub fn time_operation<F, T>(name: &str, op: F) -> T
where
    F: FnOnce() -> T,
{
    if let Some(sentry) = sentry() {
        sentry.time_operation(name, op)
    } else {
        op()
    }
}

/// Capture user feedback for the last event
#[cfg(feature = "sentry")]
pub fn capture_user_feedback(
    name: String,
    email: String,
    comments: String,
    event_id: Option<uuid::Uuid>,
) {
    if let Some(sentry) = sentry() {
        sentry.capture_user_feedback(name, email, comments, event_id);
    } else {
        warn!("Sentry not initialized - cannot capture user feedback");
    }
}

/// Start a new session for release health monitoring
#[cfg(feature = "sentry")]
pub fn start_session() {
    if let Some(sentry) = sentry() {
        sentry.start_session();
    } else {
        warn!("Sentry not initialized - cannot start session");
    }
}

/// End the current session with the given status
#[cfg(feature = "sentry")]
pub fn end_session(status: sentry::protocol::SessionStatus) {
    if let Some(sentry) = sentry() {
        sentry.end_session(status);
    } else {
        warn!("Sentry not initialized - cannot end session");
    }
}

#[cfg(not(feature = "sentry"))]
pub fn end_session(_status: ()) {
    warn!("Sentry not enabled in this build - cannot end session");
}

/// Check if monitoring is properly initialized and healthy
pub fn check_monitoring_health() -> bool {
    #[cfg(feature = "sentry")]
    {
        sentry().map_or(false, |s| s.check_health())
    }
    #[cfg(not(feature = "sentry"))]
    {
        false
    }
}

/// Get the current Sentry DSN if configured
pub fn sentry_dsn() -> Option<&'static str> {
    #[cfg(feature = "sentry")]
    {
        sentry().and_then(|s| s.dsn())
    }
    #[cfg(not(feature = "sentry"))]
    {
        None
    }
}

/// Get the current environment (production/development/etc.)
pub fn environment() -> &'static str {
    #[cfg(feature = "sentry")]
    {
        sentry().map(|s| s.environment()).unwrap_or("unknown")
    }
    #[cfg(not(feature = "sentry"))]
    {
        "unknown"
    }
}

/// Track custom events for analytics
#[cfg(feature = "sentry")]
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

// Stub implementations for when Sentry is not enabled
#[cfg(not(feature = "sentry"))]
pub fn init_monitoring() -> anyhow::Result<()> {
    info!("Sentry feature disabled at compile time");
    Ok(())
}

#[cfg(not(feature = "sentry"))]
pub fn monitor_startup() {
    // No-op when Sentry is disabled
}

#[cfg(not(feature = "sentry"))]
pub fn monitor_shutdown() {
    // No-op when Sentry is disabled
}

#[cfg(not(feature = "sentry"))]
pub fn report_critical_error(error: &anyhow::Error, context: &str) {
    // Also log to stderr for immediate visibility
    eprintln!("CRITICAL ERROR: {}: {}", context, error);
}

#[cfg(not(feature = "sentry"))]
pub fn report_error_with_tags(
    error: &anyhow::Error,
    _context: Option<&str>,
    _tags: Option<std::collections::HashMap<String, String>>,
) {
    tracing::error!("Error: {:?}", error);
}

#[cfg(not(feature = "sentry"))]
pub fn time_operation<F, T>(_name: &str, op: F) -> T
where
    F: FnOnce() -> T,
{
    op()
}

#[cfg(not(feature = "sentry"))]
pub fn capture_user_feedback(
    _name: String,
    _email: String,
    _comments: String,
    _event_id: Option<uuid::Uuid>,
) {
    warn!("Sentry not enabled in this build - cannot capture user feedback");
}

#[cfg(not(feature = "sentry"))]
pub fn start_session() {
    warn!("Sentry not enabled in this build - cannot start session");
}

#[cfg(not(feature = "sentry"))]
pub fn track_event(_event_name: &str, _properties: std::collections::HashMap<String, String>) {
    // No-op when Sentry is disabled
}
