// Comprehensive Sentry feature demonstration
// Run with: SENTRY_DSN="your-dsn" cargo run --example sentry_feature_demo --features="sentry"

use anyhow::anyhow;
use std::{collections::HashMap, thread, time::Duration};
use warp_terminal::monitoring::*;

#[cfg(feature = "sentry")]
use sentry::protocol::SessionStatus;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 === Comprehensive Sentry Feature Demo ===\n");
    
    #[cfg(feature = "sentry")]
    {
        // Initialize monitoring system
        init_monitoring()?;
        
        let has_dsn = sentry_dsn().is_some();
        
        if has_dsn {
            println!("✅ Sentry is ACTIVE - Events will be sent to dashboard");
        } else {
            println!("⚠️  Sentry is in STUB mode - Set SENTRY_DSN to send events");
        }
        
        println!("🌍 Environment: {}", environment());
        println!("📊 Health: {}", if check_monitoring_health() { "Healthy" } else { "Not Active" });
        println!();
        
        demo_lifecycle_monitoring()?;
        demo_error_reporting()?;
        demo_performance_monitoring()?;
        demo_user_context_and_tags()?;
        demo_breadcrumbs_and_context()?;
        demo_custom_events()?;
        demo_session_tracking()?;
        demo_user_feedback()?;
        
        println!("\n🎉 All Sentry features demonstrated successfully!");
        
        if has_dsn {
            println!("📱 Check your Sentry dashboard to see all captured events");
            println!("🔗 Events should appear in your project within seconds");
        } else {
            println!("💡 To see actual events, run with:");
            println!("   SENTRY_DSN=\"your-sentry-dsn\" cargo run --example sentry_feature_demo --features=\"sentry\"");
        }
    }
    
    #[cfg(not(feature = "sentry"))]
    {
        println!("❌ Sentry feature is not enabled");
        println!("   Run with: cargo run --example sentry_feature_demo --features=\"sentry\"");
        println!("   And set SENTRY_DSN environment variable to test live integration");
    }
    
    Ok(())
}

#[cfg(feature = "sentry")]
fn demo_lifecycle_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 === Application Lifecycle Monitoring ===");
    
    monitor_startup();
    println!("   ✅ Application startup monitored");
    
    thread::sleep(Duration::from_millis(100));
    
    monitor_shutdown();
    println!("   ✅ Application shutdown monitored");
    
    println!();
    Ok(())
}

#[cfg(feature = "sentry")]
fn demo_error_reporting() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 === Error Reporting Demo ===");
    
    // 1. Simple critical error
    let critical_error = anyhow!("Database connection failed unexpectedly");
    report_critical_error(&critical_error, "database_connection");
    println!("   ✅ Critical error reported");
    
    // 2. Error with custom tags
    let tagged_error = anyhow!("User authentication failed");
    let mut tags = HashMap::new();
    tags.insert("user_type".to_string(), "premium".to_string());
    tags.insert("auth_method".to_string(), "oauth".to_string());
    tags.insert("failure_reason".to_string(), "token_expired".to_string());
    
    report_error_with_tags(&tagged_error, Some("authentication"), Some(tags));
    println!("   ✅ Error with custom tags reported");
    
    // 3. Nested error chain
    let root_cause = anyhow!("Network timeout");
    let wrapped_error = root_cause.context("Failed to fetch user profile").context("Profile loading failed");
    report_error_with_tags(&wrapped_error, Some("profile_service"), None);
    println!("   ✅ Nested error chain reported");
    
    println!();
    Ok(())
}

#[cfg(feature = "sentry")]
fn demo_performance_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ === Performance Monitoring Demo ===");
    
    // 1. Time a fast operation
    let result = time_operation("database_query", || {
        thread::sleep(Duration::from_millis(50));
        "Query result"
    });
    println!("   ✅ Fast operation timed: {}", result);
    
    // 2. Time a slow operation
    let slow_result = time_operation("file_processing", || {
        thread::sleep(Duration::from_millis(200));
        42
    });
    println!("   ✅ Slow operation timed: {}", slow_result);
    
    // 3. Time a network operation
    let network_result = time_operation("api_call", || {
        thread::sleep(Duration::from_millis(150));
        serde_json::json!({"status": "success", "data": "API response"})
    });
    println!("   ✅ Network operation timed: {}", network_result);
    
    println!();
    Ok(())
}

#[cfg(feature = "sentry")]
fn demo_user_context_and_tags() -> Result<(), Box<dyn std::error::Error>> {
    println!("👤 === User Context & Tags Demo ===");
    
    // Set user context using the sentry instance directly
    if let Some(sentry) = sentry() {
        sentry.set_user(
            Some("user_12345".to_string()),
            Some("john_doe".to_string()),
            Some("john@example.com".to_string())
        );
        println!("   ✅ User context set");
        
        // Add custom tags
        sentry.set_tag("user_tier", "premium");
        sentry.set_tag("app_version", env!("CARGO_PKG_VERSION"));
        sentry.set_tag("platform", std::env::consts::OS);
        sentry.set_tag("feature_flags", "new_ui,dark_mode,ai_assist");
        println!("   ✅ Custom tags added");
        
        // Capture a message with this context
        sentry.capture_message("User performed critical action", sentry::Level::Info);
        println!("   ✅ Message with user context captured");
    }
    
    println!();
    Ok(())
}

#[cfg(feature = "sentry")]
fn demo_breadcrumbs_and_context() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍞 === Breadcrumbs & Context Demo ===");
    
    if let Some(sentry) = sentry() {
        // Add navigation breadcrumbs
        sentry.add_breadcrumb("User opened terminal", "navigation", sentry::Level::Info);
        sentry.add_breadcrumb("User ran 'ls -la' command", "user_action", sentry::Level::Info);
        sentry.add_breadcrumb("Command completed successfully", "system", sentry::Level::Info);
        println!("   ✅ Navigation breadcrumbs added");
        
        // Add debug breadcrumbs
        sentry.add_breadcrumb("Memory usage: 45MB", "system.resource", sentry::Level::Debug);
        sentry.add_breadcrumb("CPU usage: 12%", "system.resource", sentry::Level::Debug);
        println!("   ✅ System resource breadcrumbs added");
        
        // Set custom context
        let performance_context = sentry::protocol::Context::Other({
            let mut map = sentry::protocol::Map::new();
            map.insert("startup_time".to_string(), "1.2s".into());
            map.insert("memory_usage".to_string(), "45MB".into());
            map.insert("active_tabs".to_string(), 3.into());
            map.insert("plugins_loaded".to_string(), vec!["git", "docker", "kubernetes"].into());
            map
        });
        
        sentry.set_context("performance_metrics", performance_context);
        println!("   ✅ Custom performance context set");
        
        // Trigger an event to see breadcrumbs and context
        let context_error = anyhow!("This error will include all breadcrumbs and context");
        sentry.capture_error(&context_error, Some("breadcrumb_demo"));
        println!("   ✅ Error with breadcrumbs and context captured");
    }
    
    println!();
    Ok(())
}

#[cfg(feature = "sentry")]
fn demo_custom_events() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 === Custom Events & Analytics Demo ===");
    
    // Track feature usage
    let mut feature_props = HashMap::new();
    feature_props.insert("feature_name".to_string(), "auto_complete".to_string());
    feature_props.insert("trigger_type".to_string(), "tab_key".to_string());
    feature_props.insert("suggestions_shown".to_string(), "5".to_string());
    track_event("feature_used", feature_props);
    println!("   ✅ Feature usage event tracked");
    
    // Track performance metrics
    let mut perf_props = HashMap::new();
    perf_props.insert("operation".to_string(), "command_execution".to_string());
    perf_props.insert("duration_ms".to_string(), "250".to_string());
    perf_props.insert("command_type".to_string(), "git".to_string());
    track_event("performance_metric", perf_props);
    println!("   ✅ Performance metric event tracked");
    
    // Track user interaction
    let mut interaction_props = HashMap::new();
    interaction_props.insert("interaction_type".to_string(), "keyboard_shortcut".to_string());
    interaction_props.insert("shortcut".to_string(), "Cmd+K".to_string());
    interaction_props.insert("context".to_string(), "command_palette".to_string());
    track_event("user_interaction", interaction_props);
    println!("   ✅ User interaction event tracked");
    
    println!();
    Ok(())
}

#[cfg(feature = "sentry")]
fn demo_session_tracking() -> Result<(), Box<dyn std::error::Error>> {
    println!("📈 === Session Tracking Demo ===");
    
    // Start a session
    start_session();
    println!("   ✅ Session started");
    
    // Simulate some session activity
    thread::sleep(Duration::from_millis(100));
    
    // Capture some session events
    if let Some(sentry) = sentry() {
        sentry.capture_message("User session is active", sentry::Level::Info);
        println!("   ✅ Session activity captured");
    }
    
    // End session with success status
    end_session(SessionStatus::Ok);
    println!("   ✅ Session ended successfully");
    
    // Start another session and end it with an error
    start_session();
    thread::sleep(Duration::from_millis(50));
    
    // Simulate a session error
    let session_error = anyhow!("Session crashed due to memory overflow");
    report_critical_error(&session_error, "session_crash");
    
    end_session(SessionStatus::Crashed);
    println!("   ✅ Second session ended with crash status");
    
    println!();
    Ok(())
}

#[cfg(feature = "sentry")]
fn demo_user_feedback() -> Result<(), Box<dyn std::error::Error>> {
    println!("💬 === User Feedback Demo ===");
    
    // Capture user feedback
    capture_user_feedback(
        "Alice Johnson".to_string(),
        "alice@example.com".to_string(),
        "The new terminal features are great! The auto-completion is very helpful.".to_string(),
        None,
    );
    println!("   ✅ Positive user feedback captured");
    
    // Capture feedback with error context
    let feedback_error = anyhow!("Button click didn't work as expected");
    report_error_with_tags(&feedback_error, Some("ui_interaction_error"), None);
    let error_id = None; // For demo purposes
    
    capture_user_feedback(
        "Bob Wilson".to_string(),
        "bob@example.com".to_string(),
        "I found a bug - the save button doesn't respond when clicked multiple times.".to_string(),
        error_id,
    );
    println!("   ✅ Bug report feedback with error context captured");
    
    println!();
    Ok(())
}

// Stub implementations for non-sentry builds
#[cfg(not(feature = "sentry"))]
fn demo_lifecycle_monitoring() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "sentry"))]
fn demo_error_reporting() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "sentry"))]
fn demo_performance_monitoring() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "sentry"))]
fn demo_user_context_and_tags() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "sentry"))]
fn demo_breadcrumbs_and_context() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "sentry"))]
fn demo_custom_events() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "sentry"))]
fn demo_session_tracking() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
#[cfg(not(feature = "sentry"))]
fn demo_user_feedback() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
