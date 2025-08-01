// Temporary test script for Sentry integration
// Run with: cargo run --example test_sentry --features="sentry"

use anyhow::anyhow;
use std::{collections::HashMap, thread, time::Duration};
use warp_terminal::monitoring::{
    capture_user_feedback, check_monitoring_health, end_session, environment,
    report_critical_error, report_error_with_tags, sentry_dsn, start_session, time_operation,
};

#[cfg(feature = "sentry")]
use warp_terminal::monitoring::init_monitoring;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Sentry Integration Test ===\n");
    
    #[cfg(feature = "sentry")]
    {
        use sentry::protocol::SessionStatus;
        
        // Initialize monitoring
        init_monitoring()?;
        
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
    }
    
    #[cfg(not(feature = "sentry"))]
    {
        println!("❌ Sentry feature is not enabled");
        println!("   Run with: cargo run --example test_sentry --features=\"sentry\"");
        println!("   And set SENTRY_DSN environment variable to test Sentry integration");
        
        println!("\nEnvironment: {}", environment());
        println!("Sentry DSN: {}", sentry_dsn().unwrap_or("<not configured>"));
        println!("Monitoring health: {}", check_monitoring_health());
        
        // Test the stub implementations
        println!("\n1. Testing error reporting (stub)...");
        let error = anyhow!("This is a test error");
        report_critical_error(&error, "test_error_reporting");
        
        println!("2. Testing error with tags (stub)...");
        let error = anyhow!("Failed to process request");
        let mut tags = HashMap::new();
        tags.insert("component".to_string(), "test".to_string());
        report_error_with_tags(&error, Some("test_error_with_tags"), Some(tags));
        
        println!("3. Testing performance monitoring (stub)...");
        let _result = time_operation("test_operation", || {
            thread::sleep(Duration::from_millis(100));
            42
        });
        
        println!("4. Testing user feedback (stub)...");
        capture_user_feedback(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "This is a test feedback".to_string(),
            None,
        );
        
        println!("5. Testing session tracking (stub)...");
        start_session();
        thread::sleep(Duration::from_millis(200));
        end_session(()); // Use unit type for non-sentry builds
        
        println!("\n=== Test completed successfully (with stubs)! ===\n");
        println!("To test actual Sentry integration, run with --features=\"sentry\" and set SENTRY_DSN.");
    }
    
    Ok(())
}
