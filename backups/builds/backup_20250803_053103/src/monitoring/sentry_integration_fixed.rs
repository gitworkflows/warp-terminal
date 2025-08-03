use anyhow::{Context, Result};
use sentry::{
    ClientInitGuard, ClientOptions, Event, EventId, Hub, Level, SessionStatus, TransactionOrSpan,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::{info, warn};

/// Configuration for Sentry integration
#[derive(Debug, Clone)]
pub struct SentryConfig {
    /// The Sentry DSN (Data Source Name)
    pub dsn: Option<String>,
    /// Environment (e.g., "production", "staging", "development")
    pub environment: String,
    /// Release version (e.g., "myapp@1.0.0")
    pub release: String,
    /// Sample rate for error events (0.0 to 1.0)
    pub sample_rate: f32,
    /// Sample rate for performance traces (0.0 to 1.0)
    pub traces_sample_rate: f32,
    /// Enable debug mode
    pub debug: bool,
    /// Custom tags to include with all events
    pub tags: HashMap<String, String>,
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            dsn: None,
            environment: "development".to_string(),
            release: format!("warp-terminal@{}", env!("CARGO_PKG_VERSION")),
            sample_rate: 1.0,
            traces_sample_rate: 0.1,
            debug: false,
            tags: HashMap::new(),
        }
    }
}

/// Sentry integration manager
pub struct SentryIntegration {
    _guard: Option<ClientInitGuard>,
    hub: Arc<Hub>,
    config: SentryConfig,
}

impl SentryIntegration {
    /// Create a new SentryIntegration with environment-aware configuration
    pub fn with_environment_config() -> Result<Self> {
        let is_production = std::env::var("PRODUCTION")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let mut config = SentryConfig::default();

        // Load DSN from environment variable if not set
        if config.dsn.is_none() {
            config.dsn = std::env::var("SENTRY_DSN").ok();
        }

        // Set environment from environment variable if available
        if let Ok(env) = std::env::var("SENTRY_ENVIRONMENT") {
            config.environment = env;
        } else {
            config.environment = if is_production {
                "production".to_string()
            } else {
                "development".to_string()
            };
        }

        // Adjust sample rates for production
        if is_production {
            config.sample_rate = 0.1; // Sample 10% of errors in production
            config.traces_sample_rate = 0.05; // Sample 5% of traces in production
        } else {
            config.sample_rate = 1.0; // Sample all errors in development
            config.traces_sample_rate = 1.0; // Sample all traces in development
        }

        Self::new(config)
    }

    /// Initialize Sentry with the given configuration
    pub fn new(config: SentryConfig) -> Result<Self> {
        let guard = if let Some(dsn) = &config.dsn {
            info!("Initializing Sentry with DSN: {}", dsn);
            
            let mut client_options = ClientOptions {
                dsn: dsn.parse().ok(),
                release: Some(config.release.clone().into()),
                environment: Some(config.environment.clone().into()),
                send_default_pii: true,
                sample_rate: config.sample_rate,
                traces_sample_rate: config.traces_sample_rate,
                debug: config.debug,
                default_integrations: true,
                ..Default::default()
            };

            // Add custom before_send hook
            client_options.before_send = Some(Arc::new(move |mut event| {
                // Add custom tags to all events
                for (key, value) in &config.tags {
                    event.tags.insert(key.clone(), value.clone());
                }
                
                // Add platform and architecture tags
                event.tags.insert("platform".to_string(), std::env::consts::OS.to_string());
                event.tags.insert("arch".to_string(), std::env::consts::ARCH.to_string());
                
                Some(event)
            }));

            let guard = sentry::init(client_options);
            
            // Set up the global hub
            let hub = Hub::current();
            
            // Set user agent and other metadata
            hub.configure_scope(|scope| {
                scope.set_tag("app.name", "warp-terminal");
                scope.set_tag("app.version", env!("CARGO_PKG_VERSION"));
            });
            
            Some((guard, hub))
        } else {
            warn!("Sentry DSN not configured, Sentry will be disabled");
            None
        };

        if let Some((guard, hub)) = guard {
            Ok(Self {
                _guard: Some(guard),
                hub: Arc::new(hub),
                config,
            })
        } else {
            // Create a no-op hub when Sentry is disabled
            let hub = Hub::new_from_top(Hub::new(None));
            
            Ok(Self {
                _guard: None,
                hub: Arc::new(hub),
                config,
            })
        }
    }

    /// Capture an error with additional context and tags
    pub fn capture_error_with_tags(
        &self,
        error: &anyhow::Error,
        context: Option<&str>,
        tags: Option<HashMap<String, String>>,
    ) {
        self.hub.with_scope(|scope| {
            // Add context as a tag if provided
            if let Some(ctx) = context {
                scope.set_tag("context", ctx);
            }
            
            // Add custom tags if provided
            if let Some(tags) = tags {
                for (key, value) in tags {
                    scope.set_tag(key, value);
                }
            }
            
            // Capture the error
            let event_id = sentry::capture_error(error);
            info!("Captured error with ID: {}", event_id);
        });
    }

    /// Convenience method for backward compatibility
    pub fn capture_error(&self, error: &anyhow::Error, context: Option<&str>) {
        self.capture_error_with_tags(error, context, None);
    }

    /// Time an operation and report it as a transaction
    pub fn time_operation<F, T>(&self, name: &str, op: F) -> T
    where
        F: FnOnce() -> T,
    {
        let transaction = self.hub.start_transaction(
            &sentry::TransactionContext::new(name, "app.operation"),
            &Default::default(),
        );
        
        // Set the transaction on the scope
        self.hub.configure_scope(|scope| {
            scope.set_span(Some(transaction.clone().into()));
        });
        
        // Execute the operation
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(op));
        
        // Finish the transaction
        transaction.finish();
        
        // Unwrap the result and propagate panics
        match result {
            Ok(r) => r,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    /// Capture user feedback for the last event
    pub fn capture_user_feedback(
        &self,
        name: String,
        email: String,
        comments: String,
        event_id: Option<EventId>,
    ) {
        if let Some(event_id) = event_id {
            let user_feedback = sentry::UserFeedback {
                event_id,
                name,
                email,
                comments,
            };
            
            sentry::capture_user_feedback(user_feedback);
        } else if let Some(last_event_id) = self.hub.last_event_id() {
            let user_feedback = sentry::UserFeedback {
                event_id: last_event_id,
                name,
                email,
                comments,
            };
            
            sentry::capture_user_feedback(user_feedback);
        } else {
            warn!("No event ID available for user feedback");
        }
    }

    /// Start a new session for release health monitoring
    pub fn start_session(&self) {
        self.hub.start_session();
    }

    /// End the current session with the given status
    pub fn end_session(&self, status: SessionStatus) {
        self.hub.end_session();
    }

    /// Capture a message with level
    pub fn capture_message(&self, message: &str, level: Level) {
        sentry::capture_message(message, level);
    }

    /// Start a performance transaction
    pub fn start_transaction(&self, name: &str, operation: &str) -> Option<TransactionOrSpan> {
        let transaction_ctx = sentry::TransactionContext::new(name, operation);
        let transaction = self.hub.start_transaction(&transaction_ctx, &Default::default());
        Some(transaction.into())
    }

    /// Add breadcrumb for debugging
    pub fn add_breadcrumb(&self, message: &str, category: &str, level: Level) {
        self.hub.add_breadcrumb(sentry::Breadcrumb {
            message: Some(message.to_string()),
            category: Some(category.to_string()),
            level,
            ..Default::default()
        });
    }

    /// Set user context
    pub fn set_user(&self, id: Option<String>, username: Option<String>, email: Option<String>) {
        self.hub.configure_scope(|scope| {
            let mut user = sentry::User::new("");
            
            if let Some(id) = id {
                user.id = Some(id);
            }
            
            if let Some(username) = username {
                user.username = Some(username);
            }
            
            if let Some(email) = email {
                user.email = Some(email);
            }
            
            scope.set_user(Some(user));
        });
    }

    /// Set custom tag
    pub fn set_tag(&self, key: &str, value: &str) {
        self.hub.configure_scope(|scope| {
            scope.set_tag(key, value);
        });
    }

    /// Set custom context
    pub fn set_context<K, V: serde::Serialize>(&self, key: K, context: V) {
        self.hub.configure_scope(|scope| {
            scope.set_context(key, context);
        });
    }

    /// Check if Sentry is enabled
    pub fn is_enabled(&self) -> bool {
        self._guard.is_some()
    }

    /// Check if Sentry is properly initialized and active
    pub fn check_health(&self) -> bool {
        self.is_enabled() && self.hub.is_active()
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
static mut SENTRY_INSTANCE: Option<SentryIntegration> = None;
static SENTRY_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize global Sentry instance
pub fn init_sentry(config: SentryConfig) -> Result<()> {
    SENTRY_INIT.call_once(|| {
        match SentryIntegration::new(config) {
            Ok(sentry) => {
                unsafe {
                    SENTRY_INSTANCE = Some(sentry);
                }
                info!("Sentry initialized successfully");
            }
            Err(e) => {
                eprintln!("Failed to initialize Sentry: {}", e);
            }
        }
    });
    
    if let Some(sentry) = unsafe { SENTRY_INSTANCE.as_ref() } {
        if sentry.check_health() {
            info!("Sentry is active and healthy");
        } else {
            warn!("Sentry is not properly initialized");
        }
    }
    
    Ok(())
}

/// Get global Sentry instance
pub fn sentry() -> Option<&'static SentryIntegration> {
    unsafe { SENTRY_INSTANCE.as_ref() }
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
            sentry.capture_message($message, $crate::monitoring::sentry_integration::Level::Info);
        }
    };
}

#[macro_export]
macro_rules! sentry_warning {
    ($message:expr) => {
        if let Some(sentry) = $crate::monitoring::sentry_integration::sentry() {
            sentry.capture_message($message, $crate::monitoring::sentry_integration::Level::Warning);
        }
    };
}

#[macro_export]
macro_rules! sentry_breadcrumb {
    ($message:expr, $category:expr) => {
        if let Some(sentry) = $crate::monitoring::sentry_integration::sentry() {
            sentry.add_breadcrumb(
                $message,
                $category,
                $crate::monitoring::sentry_integration::Level::Info,
            );
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
    use serial_test::serial;
    use std::env;

    #[test]
    fn test_sentry_config_default() {
        let config = SentryConfig::default();
        assert!(config.dsn.is_none());
        assert_eq!(config.environment, "development");
        assert!(config.release.contains("warp-terminal@"));
        assert_eq!(config.sample_rate, 1.0);
        assert_eq!(config.traces_sample_rate, 0.1);
        assert!(!config.debug);
        assert!(config.tags.is_empty());
    }

    #[test]
    #[serial]
    fn test_environment_aware_config() {
        // Save original environment variables
        let original_dsn = env::var("SENTRY_DSN").ok();
        let original_env = env::var("SENTRY_ENVIRONMENT").ok();
        let original_prod = env::var("PRODUCTION").ok();

        // Test production environment
        env::set_var("PRODUCTION", "true");
        let config = SentryIntegration::with_environment_config()
            .unwrap()
            .config
            .clone();
        assert_eq!(config.environment, "production");
        assert_eq!(config.sample_rate, 0.1);
        assert_eq!(config.traces_sample_rate, 0.05);

        // Test development environment
        env::remove_var("PRODUCTION");
        let config = SentryIntegration::with_environment_config()
            .unwrap()
            .config
            .clone();
        assert_eq!(config.environment, "development");
        assert_eq!(config.sample_rate, 1.0);
        assert_eq!(config.traces_sample_rate, 1.0);

        // Test custom environment variable
        env::set_var("SENTRY_ENVIRONMENT", "staging");
        let config = SentryIntegration::with_environment_config()
            .unwrap()
            .config
            .clone();
        assert_eq!(config.environment, "staging");

        // Restore original environment
        if let Some(dsn) = original_dsn {
            env::set_var("SENTRY_DSN", dsn);
        } else {
            env::remove_var("SENTRY_DSN");
        }
        
        if let Some(env) = original_env {
            env::set_var("SENTRY_ENVIRONMENT", env);
        } else {
            env::remove_var("SENTRY_ENVIRONMENT");
        }
        
        if let Some(prod) = original_prod {
            env::set_var("PRODUCTION", prod);
        } else {
            env::remove_var("PRODUCTION");
        }
    }

    #[test]
    fn test_sentry_health_checks() {
        // Test with no DSN (should be disabled)
        let config = SentryConfig::default();
        let sentry = SentryIntegration::new(config).unwrap();
        assert!(!sentry.is_enabled());
        assert!(!sentry.check_health());

        // Test with invalid DSN (should still initialize but be disabled)
        let mut config = SentryConfig::default();
        config.dsn = Some("invalid-dsn".to_string());
        let sentry = SentryIntegration::new(config).unwrap();
        assert!(!sentry.is_enabled());
        assert!(!sentry.check_health());
    }

    #[test]
    fn test_error_reporting() {
        // Test with no DSN (should not panic)
        let config = SentryConfig::default();
        let sentry = SentryIntegration::new(config).unwrap();
        
        let error = anyhow::anyhow!("Test error");
        sentry.capture_error(&error, Some("test_error_reporting"));
        
        // Test with tags
        let mut tags = HashMap::new();
        tags.insert("test_key".to_string(), "test_value".to_string());
        sentry.capture_error_with_tags(&error, Some("test_with_tags"), Some(tags));
    }

    #[test]
    fn test_breadcrumbs() {
        let config = SentryConfig {
            dsn: Some("https://test@example.com/1".to_string()),
            ..Default::default()
        };
        
        let sentry = SentryIntegration::new(config).unwrap();
        sentry.add_breadcrumb("Test breadcrumb", "test", Level::Info);
        
        // In a real test with a test DSN, we could verify the breadcrumb was added
        assert!(sentry.is_enabled());
    }
}
