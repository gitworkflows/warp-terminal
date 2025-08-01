// Temporary test script for Sentry integration
// Run with: cargo run --example temp_sentry_test --features="sentry"

use anyhow::anyhow;
use std::{collections::HashMap, thread, time::Duration};
use warp_terminal::monitoring::{
    capture_user_feedback, check_monitoring_health, end_session, environment, init_monitoring,
    report_critical_error, report_error_with_tags, sentry_dsn, start_session, time_operation,
};
use sentry::protocol::SessionStatus;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize monitoring
    init_monitoring()?;
    
    println!("=== Sentry Integration Test ===\n");
    println!("Environment: {}", environment());
    println!("Sentry DSN: {}", sentry_dsn().unwrap_or("<not configured>"));
    println!("Monitoring health: {}", check_monitoring_health());
    
    // Test error reporting
    println!("\n1. Testing error reporting...");
    let error = anyhow!("This is a test error");
    report_critical_error(&error, "test_error_reporting");
    
    // Test error with tags
    println!("2. Testing error with tags...");
    let error = anyhow!("Failed to process request");
    let mut tags = HashMap::new();
    tags.insert("component".to_string(), "test".to_string());
    report_error_with_tags(&error, Some("test_error_with_tags"), Some(tags));
    
    // Test performance monitoring
    println!("3. Testing performance monitoring...");
    let _result = time_operation("test_operation", || {
        thread::sleep(Duration::from_millis(100));
        42
    });
    
    // Test user feedback
    println!("4. Testing user feedback...");
    capture_user_feedback(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "This is a test feedback".to_string(),
        None,
    );
    
    // Test session tracking
    println!("5. Testing session tracking...");
    start_session();
    thread::sleep(Duration::from_millis(200));
    end_session(SessionStatus::Ok);
    
    println!("\n=== Test completed successfully! ===\n");
    println!("Check your Sentry dashboard to verify all events were captured correctly.");
    
    Ok(())
}
