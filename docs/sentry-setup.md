# Sentry Integration Setup

This document explains how to set up and configure Sentry error reporting for the Warp Terminal application.

## Overview

Sentry has been integrated into the Warp Terminal to provide:

- **Error Reporting**: Automatic capture of panics and errors
- **Performance Monitoring**: Transaction tracing and performance insights
- **Custom Event Tracking**: User interaction and feature usage analytics
- **Breadcrumbs**: Detailed context for debugging issues
- **Release Tracking**: Version-based error tracking and performance regression detection

## Configuration

### Environment Variables

Set the following environment variables to enable Sentry:

```bash
# Required: Your Sentry DSN (Data Source Name)
export SENTRY_DSN="https://2c4bb2382fa2f89065882214bd82060e@o4507943605305344.ingest.de.sentry.io/4509765617451088"

# Optional: Environment name (defaults to "development")
export SENTRY_ENVIRONMENT="production"  # or "staging", "development"
```

### SDK Configuration

The Sentry SDK is configured with the following default options:

- **Release Tracking**: Automatically set using `release_name!()` macro
- **PII Collection**: Enabled by default (captures IPs, etc.)
- **Sample Rates**: 
  - Error events: 100% (configurable via `sample_rate`)
  - Performance traces: 10% (configurable via `traces_sample_rate`)
- **Debug Mode**: Enabled in debug builds

### Customizing Configuration

You can customize the Sentry configuration by creating a `SentryConfig` instance:

```rust
use warp_terminal::monitoring::SentryConfig;

let config = SentryConfig {
    dsn: Some(std::env::var("SENTRY_DSN").ok()),
    environment: "production".to_string(),
    release: format!("warp-terminal@{}", env!("CARGO_PKG_VERSION")),
    traces_sample_rate: 0.1,  // 10% of transactions
    sample_rate: 1.0,        // 100% of errors
    debug: cfg!(debug_assertions),
    tags: std::collections::HashMap::new(),
};
```

### Sentry Project Setup

1. **Create a Sentry Project**:
   - Go to [sentry.io](https://sentry.io)
   - Create a new project for "Rust"
   - Copy the DSN from the project settings

2. **Configure Release Tracking**:
   - Releases are automatically set to `warp-terminal@{version}` format
   - Version is pulled from `Cargo.toml`

3. **Set Up Alerts** (Optional):
   - Configure alert rules for critical errors
   - Set up performance threshold alerts
   - Enable weekly/monthly performance reports

## Features

### Error Reporting

The integration automatically captures:

- **Panics**: All application panics are reported with full stack traces
- **Critical Errors**: Errors marked as critical using `report_critical_error()`
- **Context**: Terminal session information, user interaction context

### Performance Monitoring

- **Startup Tracking**: Application initialization performance
- **Command Execution**: Individual command performance metrics
- **UI Rendering**: Performance impact of UI operations
- **Plugin Operations**: Plugin loading and execution performance

### Custom Events

Track user interactions and feature usage:

```rust
use warp_terminal::monitoring::track_event;

// Track feature usage
let mut properties = HashMap::new();
properties.insert("feature".to_string(), "command_palette".to_string());
properties.insert("action".to_string(), "opened".to_string());
track_event("feature_used", properties);
```

### Breadcrumbs

Automatic breadcrumbs are added for:

- Application lifecycle events (startup, shutdown)
- Terminal state changes
- Command executions
- Plugin operations
- Error occurrences

### Manual Error Reporting

Use the convenience macros for manual error reporting:

```rust
use warp_terminal::{sentry_error, sentry_warning, sentry_info, sentry_breadcrumb};

// Report an error with context
let error = anyhow::anyhow!("Something went wrong");
sentry_error!(&error, "processing_command");

// Add informational message
sentry_info!("Command executed successfully");

// Add breadcrumb for debugging
sentry_breadcrumb!("User opened settings", "ui", sentry::Level::Info);
```

## Configuration Options

### Sample Rates

The default configuration uses:

- **Error Sample Rate**: 100% (all errors are reported)
- **Performance Sample Rate**: 10% (1 in 10 transactions are traced)

### Custom Tags

All events are automatically tagged with:

- **Platform**: Operating system (macOS, Linux, Windows)
- **Architecture**: CPU architecture (x86_64, arm64, etc.)
- **Version**: Application version from Cargo.toml
- **Environment**: Development, staging, or production

### Context Information

Each event includes:

- **App Info**: Application name, version, build timestamp
- **Terminal Session**: Unique session identifier
- **User Context**: If available (can be set via `set_user()`)

## Privacy and Security

### Data Collected

Sentry integration collects:

- Error messages and stack traces
- Performance metrics (timing data)
- User interaction events (feature usage)
- System information (OS, architecture)
- Application version and build information

### Data NOT Collected

- Command line inputs or outputs
- File contents or paths
- Personal information
- Network requests or responses
- Sensitive environment variables

### Local Development

When `SENTRY_DSN` is not set, all Sentry functionality is disabled:

- No data is sent to external servers
- Error reporting falls back to local logging
- Performance monitoring is disabled
- Events are not tracked

## Testing the Integration

### Verify Sentry is Working

1. **Check Logs**: Look for "Sentry initialized successfully" in application logs
2. **Trigger Test Error**: Use the built-in error testing (if available)
3. **Check Sentry Dashboard**: Verify events appear in your Sentry project

### Debug Mode

Enable debug output by setting the environment to development:

```bash
export SENTRY_ENVIRONMENT="development"
```

This enables verbose Sentry logging to help diagnose integration issues.

## Troubleshooting

### Common Issues

1. **No Events in Sentry**:
   - Verify `SENTRY_DSN` is set correctly
   - Check network connectivity
   - Ensure the DSN is for the correct project

2. **Performance Data Missing**:
   - Performance monitoring has a 10% sample rate by default
   - Increase sample rate for testing: modify `traces_sample_rate` in code

3. **Missing Context**:
   - Custom context should be set using the Sentry integration methods
   - Check that breadcrumbs are being added before errors occur

### Error Investigation

When investigating errors in Sentry:

1. **Check Breadcrumbs**: Review the sequence of events leading to the error
2. **Examine Context**: Look at terminal session info and user actions
3. **Stack Trace**: Use the full stack trace to identify the error source
4. **Release Comparison**: Compare error rates across application versions

## Best Practices

### Error Reporting

- Use `report_critical_error()` for errors that need immediate attention
- Add context to errors using the second parameter
- Use appropriate Sentry levels (Error, Warning, Info)

### Performance Monitoring

- Wrap long-running operations in transactions
- Use meaningful transaction names (e.g., "command.execution", "plugin.load")
- Monitor performance regressions across releases

### Event Tracking

- Track user interactions that indicate feature usage
- Use consistent event naming conventions
- Include relevant properties for analytics

### Privacy

- Never include sensitive information in error reports
- Review custom context before adding to events
- Follow data retention policies for your organization

## Support

For issues with the Sentry integration:

1. Check the [Sentry Rust SDK documentation](https://docs.sentry.io/platforms/rust/)
2. Review this integration code in `src/monitoring/`
3. Create an issue in the project repository with:
   - Environment details
   - Sentry configuration (without DSN)
   - Error logs or symptoms
