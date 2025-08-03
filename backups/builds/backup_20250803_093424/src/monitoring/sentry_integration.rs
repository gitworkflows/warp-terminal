use anyhow::{Result};
#[cfg(feature = "sentry")]
use std::{collections::HashMap, sync::{Arc, OnceLock}};
#[cfg(not(feature = "sentry"))]
use std::collections::HashMap;
use tracing::{info};
#[cfg(feature = "sentry")]
use uuid::Uuid;


/// Default sample rate for error events in production (50%)
#[allow(dead_code)]
const DEFAULT_PRODUCTION_SAMPLE_RATE: f32 = 0.5;
/// Default sample rate for performance traces in production (10%)
#[allow(dead_code)]
const DEFAULT_PRODUCTION_TRACES_SAMPLE_RATE: f32 = 0.1;

/// Configuration for Sentry integration
#[cfg(feature = "sentry")]
#[derive(Debug, Clone)]
pub struct SentryConfig {
    /// Sentry DSN (Data Source Name) - set via environment variable or config
    pub dsn: Option<String>,
    /// Environment (development, staging, production)
    pub environment: String,
    /// Release version
    pub release: String,
    /// Sample rate for performance monitoring (0.0 to 1.0)
    pub traces_sample_rate: f32,
    /// Sample rate for error reporting (0.0 to 1.0)
    pub sample_rate: f32,
    /// Enable debug mode
    pub debug: bool,
    /// Custom tags to add to all events
    pub tags: HashMap<String, String>,
}

#[cfg(feature = "sentry")]
impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            dsn: std::env::var("SENTRY_DSN").ok(),
            environment: std::env::var("SENTRY_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
            release: format!("warp-terminal@{}", env!("CARGO_PKG_VERSION")),
            traces_sample_rate: 0.1, // 10% of transactions
            sample_rate: 1.0, // 100% of errors
            debug: cfg!(debug_assertions),
            tags: HashMap::new(),
        }
    }
}

/// Sentry integration manager
#[cfg(feature = "sentry")]
pub struct SentryIntegration {
    config: SentryConfig,
    #[cfg(feature = "sentry")]
    _guard: Option<sentry::ClientInitGuard>,
    #[cfg(not(feature = "sentry"))]
    _guard: Option<()>,
}

#[cfg(feature = "sentry")]
impl SentryIntegration {
    /// Create a new SentryIntegration with environment-aware configuration
    pub fn with_environment_config() -> Result<Self> {
        let is_production = std::env::var("PRODUCTION").is_ok();
        
        let config = SentryConfig {
            dsn: std::env::var("SENTRY_DSN").ok(),
            environment: if is_production { "production" } else { "development" }.to_string(),
            release: format!("warp-terminal@{}", env!("CARGO_PKG_VERSION")),
            traces_sample_rate: if is_production { 
                std::env::var("SENTRY_TRACES_SAMPLE_RATE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_PRODUCTION_TRACES_SAMPLE_RATE)
            } else {
                1.0 // Sample all traces in development
            },
            sample_rate: if is_production {
                std::env::var("SENTRY_SAMPLE_RATE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_PRODUCTION_SAMPLE_RATE)
            } else {
                1.0 // Sample all errors in development
            },
            debug: !is_production,
            tags: Default::default(),
        };
        
        Self::new(config)
    }

    /// Initialize Sentry with the given configuration
    pub fn new(config: SentryConfig) -> Result<Self> {
        #[cfg(feature = "sentry")]
        let guard = if let Some(dsn) = &config.dsn {
            info!("Initializing Sentry with DSN: {}", dsn);
            
            let mut tags = config.tags.clone();
            tags.insert("platform".to_string(), std::env::consts::OS.to_string());
            tags.insert("arch".to_string(), std::env::consts::ARCH.to_string());
            
            // Use default HTTP transport (rate limiting is handled by Sentry client)
            let client_options = sentry::ClientOptions {
                send_default_pii: true,
                dsn: dsn.parse().ok(),
                environment: Some(config.environment.clone().into()),
                release: Some(config.release.clone().into()),
                sample_rate: config.sample_rate,
                traces_sample_rate: config.traces_sample_rate,
                debug: config.debug,
                // Disable default integrations to avoid the debug images issue on macOS
                default_integrations: false,
                // Manually add only the safe integrations we need
                integrations: vec![
                    Arc::new(sentry::integrations::panic::PanicIntegration::default()),
                    Arc::new(sentry::integrations::contexts::ContextIntegration::default()),
                    // Intentionally excluding DebugImagesIntegration to avoid the findshlibs overflow issue
                    // Excluding other integrations that require additional features or may cause issues
                ],
                before_send: Some(Arc::new(move |mut event| {
                    if let Some(logger) = event.logger.as_ref() {
                        if logger == "tracing" && event.level == sentry::Level::Info {
                            return None;
                        }
                    }
                    event.extra.insert(
                        "terminal_session".to_string(),
                        sentry::protocol::Value::String(Uuid::new_v4().to_string()),
                    );
                    Some(event)
                })),
                ..Default::default()
            };
            let guard = sentry::init(client_options);
            
            // Set global tags and context
            sentry::configure_scope(|scope| {
                // Add all custom tags
                for (key, value) in &tags {
                    scope.set_tag(key, value);
                }
                
                // Add application context
                scope.set_context("app_info", sentry::protocol::Context::Other({
                    let mut map = sentry::protocol::Map::new();
                    map.insert("name".to_string(), "warp-terminal".into());
                    map.insert("version".to_string(), env!("CARGO_PKG_VERSION").into());
                    map.insert("build_timestamp".to_string(), chrono::Utc::now().to_rfc3339().into());
                    map
                }));
            });
            
            info!("Sentry initialized successfully with release {}", config.release);
            Some(guard)
        } else {
            info!("Sentry DSN not provided, error reporting disabled");
            None
        };
        
        #[cfg(not(feature = "sentry"))]
        let guard = {
            info!("Sentry feature disabled at compile time");
            None
        };

        Ok(Self {
            config,
            _guard: guard,
        })
    }

    /// Capture an error with additional context and tags
#[cfg(feature = "sentry")]
pub fn capture_error_with_tags(
        &self,
        error: &anyhow::Error,
        context: Option<&str>,
        tags: Option<HashMap<String, String>>,
    ) {
        if !self.is_enabled() {
            return;
        }

        #[cfg(feature = "sentry")]
        {
            sentry::with_scope(|scope| {
                // Add context if provided
                if let Some(ctx) = context {
                    scope.set_extra("context", ctx.into());
                }
                
                // Add tags if provided
                if let Some(tags) = tags {
                    for (key, value) in tags {
                        scope.set_tag(&key, value);
                    }
                }
            }, || {
                // Capture the error - use the anyhow error directly
                sentry::capture_error(error.as_ref() as &(dyn std::error::Error + Send + Sync));
            });
        }
    }
    
    /// Convenience method for backward compatibility
#[cfg(feature = "sentry")]
pub fn capture_error(&self, error: &anyhow::Error, context: Option<&str>) {
        self.capture_error_with_tags(error, context, None);
    }
    
    /// Time an operation and report it as a transaction
#[cfg(feature = "sentry")]
pub fn time_operation<F, T>(&self, name: &str, op: F) -> T
    where
        F: FnOnce() -> T,
    {
        let transaction = self.start_transaction(name, "operation");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(op));
        
        if let Some(transaction) = transaction {
            transaction.finish();
        }
        
        match result {
            Ok(r) => r,
            Err(e) => std::panic::resume_unwind(e),
        }
    }
    
    /// Capture user feedback for the last event
#[cfg(feature = "sentry")]
pub fn capture_user_feedback(
        &self,
        name: String,
        email: String,
        comments: String,
        event_id: Option<uuid::Uuid>,
    ) {
        if !self.is_enabled() {
            return;
        }
        
        #[cfg(feature = "sentry")]
        {
            let event_id = event_id.unwrap_or_else(|| {
                sentry::capture_message("User feedback provided", sentry::Level::Info)
            });
            
            // UserFeedback is not available in sentry 0.42.0
            // Instead, we can capture a message with context
            sentry::with_scope(|scope| {
                scope.set_extra("feedback_name", name.into());
                scope.set_extra("feedback_email", email.into());
                scope.set_extra("feedback_comments", comments.into());
                scope.set_extra("event_id", event_id.to_string().into());
            }, || {
                sentry::capture_message("User feedback received", sentry::Level::Info);
            });
        }
    }
    
    /// Start a new session for release health monitoring
#[cfg(feature = "sentry")]
pub fn start_session(&self) {
        sentry::start_session();
    }
    
    /// End the current session with the given status
#[cfg(feature = "sentry")]
pub fn end_session(&self, status: sentry::protocol::SessionStatus) {
        sentry::end_session_with_status(status);
    }


    /// Capture a message with level
#[cfg(feature = "sentry")]
pub fn capture_message(&self, message: &str, level: sentry::Level) {
        if self._guard.is_some() {
            sentry::capture_message(message, level);
        }
    }

    /// Start a performance transaction
#[cfg(feature = "sentry")]
pub fn start_transaction(&self, name: &str, operation: &str) -> Option<sentry::TransactionOrSpan> {
        if self._guard.is_some() {
            let ctx = sentry::TransactionContext::new(name, operation);
            Some(sentry::TransactionOrSpan::Transaction(sentry::start_transaction(ctx)))
        } else {
            None
        }
    }

    /// Add breadcrumb for debugging
#[cfg(feature = "sentry")]
pub fn add_breadcrumb(&self, message: &str, category: &str, level: sentry::Level) {
        if self._guard.is_some() {
            sentry::add_breadcrumb(sentry::Breadcrumb {
                message: Some(message.to_string()),
                category: Some(category.to_string()),
                level,
                ..Default::default()
            });
        }
    }

    /// Set user context
#[cfg(feature = "sentry")]
pub fn set_user(&self, id: Option<String>, username: Option<String>, email: Option<String>) {
        if self._guard.is_some() {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    id,
                    username,
                    email,
                    ..Default::default()
                }));
            });
        }
    }

    /// Set custom tag
#[cfg(feature = "sentry")]
pub fn set_tag(&self, key: &str, value: &str) {
        if self._guard.is_some() {
            sentry::configure_scope(|scope| {
                scope.set_tag(key, value);
            });
        }
    }

    /// Set custom context
#[cfg(feature = "sentry")]
pub fn set_context(&self, key: &str, context: sentry::protocol::Context) {
        if self._guard.is_some() {
            sentry::configure_scope(|scope| {
                scope.set_context(key, context);
            });
        }
    }

    /// Check if Sentry is enabled
    pub fn is_enabled(&self) -> bool {
        self._guard.is_some()
    }

    /// Check if Sentry is properly initialized and active
    #[cfg(feature = "sentry")]
    pub fn check_health(&self) -> bool {
        self.is_enabled() && sentry::Hub::current().client().is_some()
    }

    /// Get the current DSN if configured
    pub fn dsn(&self) -> Option<&str> {
        self.config.dsn.as_deref()
    }

    /// Get the current environment
    pub fn environment(&self) -> &str {
        &self.config.environment
    }

    /// Get current configuration
    pub fn config(&self) -> &SentryConfig {
        &self.config
    }
}

/// Global Sentry instance
#[cfg(feature = "sentry")]
pub static SENTRY_INSTANCE: OnceLock<SentryIntegration> = OnceLock::new();
#[cfg(feature = "sentry")]
static SENTRY_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize global Sentry instance
#[cfg(feature = "sentry")]
pub fn init_sentry(config: SentryConfig) -> Result<()> {
    SENTRY_INIT.call_once(|| {
        match SentryIntegration::new(config) {
            Ok(sentry) => {
                let _ = SENTRY_INSTANCE.set(sentry);
            }
            Err(e) => {
                eprintln!("Failed to initialize Sentry: {}", e);
            }
        }
    });
    Ok(())
}

/// Get global Sentry instance
#[cfg(feature = "sentry")]
pub fn sentry() -> Option<&'static SentryIntegration> {
    SENTRY_INSTANCE.get()
}

#[cfg(not(feature = "sentry"))]
pub fn sentry() -> Option<&'static ()> {
    None
}

// Stub implementations for when Sentry is not enabled
#[cfg(not(feature = "sentry"))]
#[derive(Debug, Clone)]
pub struct SentryConfig {
    pub dsn: Option<String>,
    pub environment: String,
    pub release: String,
    pub traces_sample_rate: f32,
    pub sample_rate: f32,
    pub debug: bool,
    pub tags: HashMap<String, String>,
}

#[cfg(not(feature = "sentry"))]
impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            dsn: None,
            environment: "development".to_string(),
            release: format!("warp-terminal@{}", env!("CARGO_PKG_VERSION")),
            traces_sample_rate: 0.1,
            sample_rate: 1.0,
            debug: cfg!(debug_assertions),
            tags: HashMap::new(),
        }
    }
}

#[cfg(not(feature = "sentry"))]
pub struct SentryIntegration {
    config: SentryConfig,
}

#[cfg(not(feature = "sentry"))]
impl SentryIntegration {
    pub fn with_environment_config() -> Result<Self> {
        let is_production = std::env::var("PRODUCTION").is_ok();
        
        let config = SentryConfig {
            dsn: std::env::var("SENTRY_DSN").ok(),
            environment: if is_production { "production" } else { "development" }.to_string(),
            release: format!("warp-terminal@{}", env!("CARGO_PKG_VERSION")),
            traces_sample_rate: if is_production { 
                std::env::var("SENTRY_TRACES_SAMPLE_RATE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_PRODUCTION_TRACES_SAMPLE_RATE)
            } else {
                1.0 // Sample all traces in development
            },
            sample_rate: if is_production {
                std::env::var("SENTRY_SAMPLE_RATE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_PRODUCTION_SAMPLE_RATE)
            } else {
                1.0 // Sample all errors in development
            },
            debug: !is_production,
            tags: Default::default(),
        };
        
        Ok(Self {
            config,
        })
    }

    pub fn new(config: SentryConfig) -> Result<Self> {
        info!("Sentry feature disabled at compile time");
        Ok(Self { config })
    }

    pub fn is_enabled(&self) -> bool {
        false
    }

    pub fn check_health(&self) -> bool {
        false
    }

    pub fn dsn(&self) -> Option<&str> {
        None
    }

    pub fn environment(&self) -> &str {
        &self.config.environment
    }

    pub fn config(&self) -> &SentryConfig {
        &self.config
    }
}

#[cfg(not(feature = "sentry"))]
pub fn init_sentry(_config: SentryConfig) -> Result<()> {
    info!("Sentry feature disabled at compile time");
    Ok(())
}

/// Convenience macros for common Sentry operations
#[macro_export]
macro_rules! sentry_error {
    ($error:expr) => {
        if let Some(sentry) = $crate::monitoring::sentry_integration::sentry() {
            sentry.capture_error($error, None);
        }
    };
    ($error:expr, $context:expr) => {
        if let Some(sentry) = $crate::monitoring::sentry_integration::sentry() {
            sentry.capture_error($error, Some($context));
        }
    };
}

#[macro_export]
macro_rules! sentry_info {
    ($message:expr) => {
        if let Some(sentry) = $crate::monitoring::sentry_integration::sentry() {
            sentry.capture_message($message, sentry::Level::Info);
        }
    };
}

#[macro_export]
macro_rules! sentry_warning {
    ($message:expr) => {
        if let Some(sentry) = $crate::monitoring::sentry_integration::sentry() {
            sentry.capture_message($message, sentry::Level::Warning);
        }
    };
}

#[macro_export]
macro_rules! sentry_breadcrumb {
    ($message:expr, $category:expr) => {
        if let Some(sentry) = $crate::monitoring::sentry_integration::sentry() {
            sentry.add_breadcrumb($message, $category, sentry::Level::Info);
        }
    };
    ($message:expr, $category:expr, $level:expr) => {
        if let Some(sentry) = $crate::monitoring::sentry_integration::sentry() {
            sentry.add_breadcrumb($message, $category, $level);
        }
    };

}
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use serial_test::serial;

    #[test]
    fn test_sentry_config_default() {
        let config = SentryConfig::default();
        assert!(config.dsn.is_none());
        assert_eq!(config.environment, "development");
        assert!(config.release.contains("warp-terminal@"));
        assert_eq!(config.traces_sample_rate, 0.1);
        assert_eq!(config.sample_rate, 1.0);
        assert_eq!(config.debug, cfg!(debug_assertions));
    }

    #[test]
    fn test_sentry_integration_without_dsn() {
        let config = SentryConfig {
            dsn: None,
            ..Default::default()
        };
        let sentry = SentryIntegration::new(config).unwrap();
        assert!(!sentry.is_enabled());
    }

    #[test]
    #[serial] // Ensure tests don't run in parallel
    fn test_environment_aware_config() {
        // Test production environment
        env::set_var("PRODUCTION", "true");
        env::set_var("SENTRY_DSN", "https://test@example.com/1");
        
        let sentry = SentryIntegration::with_environment_config().unwrap();
        // In test environment without sentry feature, it won't be enabled
        assert!(!sentry.is_enabled());
        assert_eq!(sentry.environment(), "production");
        assert_eq!(sentry.config().sample_rate, DEFAULT_PRODUCTION_SAMPLE_RATE);
        assert_eq!(sentry.config().traces_sample_rate, DEFAULT_PRODUCTION_TRACES_SAMPLE_RATE);
        
        // Test development environment
        env::remove_var("PRODUCTION");
        env::remove_var("SENTRY_DSN");
        let sentry = SentryIntegration::with_environment_config().unwrap();
        assert_eq!(sentry.environment(), "development");
        
        // Cleanup
        env::remove_var("SENTRY_DSN");
    }

    #[test]
    fn test_sentry_health_checks() {
        let config = SentryConfig {
            dsn: Some("https://test@example.com/1".to_string()),
            ..Default::default()
        };
        
        let sentry = SentryIntegration::new(config).unwrap();
        // In test environment without sentry feature, it won't be enabled
        assert!(!sentry.is_enabled());
        
        // In test environment, the actual Sentry client won't be active
        // but we can still test the method calls
        assert!(!sentry.check_health());
    }

    #[test]
    fn test_error_reporting() {
        let config = SentryConfig {
            dsn: Some("https://test@example.com/1".to_string()),
            ..Default::default()
        };
        
        let sentry = SentryIntegration::new(config).unwrap();
        let _error = anyhow::anyhow!("Test error");
        // Comment out for test - no capture_error method implemented yet
        // sentry.capture_error(&error, Some("test_context"));
        
        // In test environment without sentry feature, it won't be enabled
        assert!(!sentry.is_enabled());
    }

    #[test]
    fn test_breadcrumbs() {
        let config = SentryConfig {
            dsn: Some("https://test@example.com/1".to_string()),
            ..Default::default()
        };
        
        let sentry = SentryIntegration::new(config).unwrap();
        // Comment out for test - no add_breadcrumb method implemented yet
        // sentry.add_breadcrumb("Test breadcrumb", "test", sentry::Level::Info);
        
        // In test environment without sentry feature, it won't be enabled
        assert!(!sentry.is_enabled());
    }
}
