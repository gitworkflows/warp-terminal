# Warp Terminal Sentry Integration Examples

This directory contains examples demonstrating how to use Sentry error reporting and monitoring integration in Warp Terminal.

## Examples

### 1. `simple_sentry_test.rs` - Direct Sentry API Usage

This example shows how to use the Sentry SDK directly with conditional compilation. It demonstrates:

- Safe Sentry initialization without debug images (avoiding macOS issues)
- Message capture
- Error capture  
- Breadcrumb tracking
- User context setting
- Custom tags

**Usage:**

```bash
# Without Sentry feature (shows graceful degradation)
cargo run --example simple_sentry_test

# With Sentry feature but no DSN (shows initialization without DSN)
cargo run --example simple_sentry_test --features="sentry"

# With Sentry feature and DSN (full functionality)
SENTRY_DSN="your-dsn-here" cargo run --example simple_sentry_test --features="sentry"
```

### 2. `test_sentry.rs` - Warp Terminal Monitoring Integration

This example demonstrates the higher-level monitoring integration built into Warp Terminal:

- Initialization via the monitoring module
- Error reporting with context and tags
- Performance monitoring
- User feedback capture
- Session tracking
- Health checks

**Usage:**

```bash
# Without Sentry feature (shows stub implementations)
cargo run --example test_sentry

# With Sentry feature but no DSN 
cargo run --example test_sentry --features="sentry"

# With Sentry feature and DSN (full functionality)
SENTRY_DSN="your-dsn-here" cargo run --example test_sentry --features="sentry"
```

## Conditional Compilation

Both examples use conditional compilation with the `sentry` feature flag:

- **Without `--features="sentry"`**: Examples compile and run but show helpful messages about the missing feature
- **With `--features="sentry"`**: Full Sentry functionality is available

This approach ensures that:
- The codebase compiles without Sentry dependencies when the feature is disabled
- Runtime behavior gracefully degrades when Sentry is not configured
- Users get clear instructions on how to enable full functionality

## Environment Variables

- `SENTRY_DSN`: Your Sentry Data Source Name (required for actual error reporting)
- `SENTRY_ENVIRONMENT`: Environment name (defaults to "development")
- `PRODUCTION`: If set, enables production-specific sample rates
- `SENTRY_SAMPLE_RATE`: Custom error sampling rate for production
- `SENTRY_TRACES_SAMPLE_RATE`: Custom trace sampling rate for production

## Features Demonstrated

### Error Safety
- Debug images integration disabled to avoid macOS stack overflow issues
- Only safe Sentry integrations are enabled
- Graceful fallback when Sentry is not available

### Performance Monitoring
- Transaction tracking
- Custom performance measurements
- Sampling rate configuration

### Context Management
- User context setting
- Custom tags and metadata
- Breadcrumb tracking for debugging

### Production Readiness
- Environment-aware configuration
- Configurable sampling rates
- Health monitoring
- Session tracking for release health

## Integration Patterns

The examples show two main integration patterns:

1. **Direct Integration** (`simple_sentry_test.rs`): Use Sentry SDK directly with conditional compilation
2. **Abstracted Integration** (`test_sentry.rs`): Use Warp Terminal's monitoring module for higher-level APIs

Choose the pattern that best fits your use case:
- Use direct integration for fine-grained control
- Use abstracted integration for consistency with Warp Terminal patterns
