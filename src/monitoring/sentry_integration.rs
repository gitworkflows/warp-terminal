use anyhow::Result;
use sentry::{ClientOptions, IntoDsn};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

/// Configuration for Sentry integration
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
pub struct SentryIntegration {
    config: SentryConfig,
    _guard: Option<sentry::ClientInitGuard>,
}

impl SentryIntegration {
    /// Initialize Sentry with the given configuration
    pub fn new(config: SentryConfig) -> Result<Self> {
        let guard = if let Some(dsn) = &config.dsn {
            info!("Initializing Sentry with DSN: {}", dsn);
            
            let mut tags = config.tags.clone();
            tags.insert("platform".to_string(), std::env::consts::OS.to_string());
            tags.insert("arch".to_string(), std::env::consts::ARCH.to_string());
            
            let options = ClientOptions {
                dsn: dsn.clone().into_dsn()?,
                environment: Some(config.environment.clone().into()),
                release: Some(config.release.clone().into()),
                sample_rate: config.sample_rate,
                traces_sample_rate: config.traces_sample_rate,
                debug: config.debug,
                default_integrations: true,
                before_send: Some(std::sync::Arc::new(|mut event| {
                    // Add custom context to all events
                    event.extra.insert(
                        "terminal_session".to_string(),
                        sentry::protocol::Value::String(Uuid::new_v4().to_string()),
                    );
                    Some(event)
                })),
                ..Default::default()
            };

            let guard = sentry::init(options);
            
            // Set global tags
            sentry::configure_scope(|scope| {
                for (key, value) in &tags {
                    scope.set_tag(key, value);
                }
                scope.set_context("app_info", sentry::protocol::Context::Other({
                    let mut map = sentry::protocol::Map::new();
                    map.insert("name".to_string(), "warp-terminal".into());
                    map.insert("version".to_string(), env!("CARGO_PKG_VERSION").into());
                    map.insert("build_timestamp".to_string(), chrono::Utc::now().to_rfc3339().into());
                    map
                }));
            });
            
            info!("Sentry initialized successfully");
            Some(guard)
        } else {
            info!("Sentry DSN not provided, error reporting disabled");
            None
        };

        Ok(Self {
            config,
            _guard: guard,
        })
    }

    /// Capture an error with additional context
    pub fn capture_error(&self, error: &anyhow::Error, context: Option<&str>) {
        if self._guard.is_some() {
            sentry::configure_scope(|scope| {
                if let Some(ctx) = context {
                    scope.set_extra("error_context", ctx.into());
                }
                scope.set_level(Some(sentry::Level::Error));
            });
            // Convert anyhow::Error to std::error::Error for Sentry
            let std_error = error.as_ref() as &dyn std::error::Error;
            sentry::capture_error(std_error);
        }
    }

    /// Capture a message with level
    pub fn capture_message(&self, message: &str, level: sentry::Level) {
        if self._guard.is_some() {
            sentry::capture_message(message, level);
        }
    }

    /// Start a performance transaction
    pub fn start_transaction(&self, name: &str, operation: &str) -> Option<sentry::TransactionOrSpan> {
        if self._guard.is_some() {
            let ctx = sentry::TransactionContext::new(name, operation);
            Some(sentry::TransactionOrSpan::Transaction(sentry::start_transaction(ctx)))
        } else {
            None
        }
    }

    /// Add breadcrumb for debugging
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
    pub fn set_tag(&self, key: &str, value: &str) {
        if self._guard.is_some() {
            sentry::configure_scope(|scope| {
                scope.set_tag(key, value);
            });
        }
    }

    /// Set custom context
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
            }
            Err(e) => {
                eprintln!("Failed to initialize Sentry: {}", e);
            }
        }
    });
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

    #[test]
    fn test_sentry_config_default() {
        let config = SentryConfig::default();
        assert_eq!(config.environment, "development");
        assert!(config.release.starts_with("warp-terminal@"));
        assert_eq!(config.sample_rate, 1.0);
        assert_eq!(config.traces_sample_rate, 0.1);
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
}
