// Simple Sentry test to verify the integration works without debug images
// Run with: SENTRY_DSN="your-dsn" cargo run --example simple_sentry_test --features="sentry"

fn main() {
    println!("=== Simple Sentry Test ===");
    
    #[cfg(feature = "sentry")]
    {
        use std::sync::Arc;
        
        // Initialize Sentry with minimal safe configuration (no debug images)
        let _guard = sentry::init(sentry::ClientOptions {
            dsn: std::env::var("SENTRY_DSN").ok().and_then(|dsn| dsn.parse().ok()),
            release: Some("warp-terminal@test".into()),
            environment: Some("test".into()),
            send_default_pii: false, // Keep privacy-focused
            sample_rate: 1.0,
            traces_sample_rate: 1.0,
            debug: true,
            // Disable default integrations to avoid debug images issue
            default_integrations: false,
            // Add only the safe integrations we need
            integrations: vec![
                Arc::new(sentry::integrations::panic::PanicIntegration::default()),
                Arc::new(sentry::integrations::contexts::ContextIntegration::default()),
            ],
            ..Default::default()
        });
        
        if let Some(dsn) = std::env::var("SENTRY_DSN").ok() {
            println!("✅ Sentry initialized with DSN: {}", dsn);
            
            // Test 1: Capture a simple message
            println!("📝 Test 1: Capturing a message...");
            let event_id = sentry::capture_message("Simple Sentry test message", sentry::Level::Info);
            println!("   Message captured with ID: {}", event_id);
            
            // Test 2: Capture an error
            println!("🔥 Test 2: Capturing an error...");
            let error = std::io::Error::new(std::io::ErrorKind::Other, "Test error for Sentry");
            let event_id = sentry::capture_error(&error);
            println!("   Error captured with ID: {}", event_id);
            
            // Test 3: Add breadcrumb
            println!("🍞 Test 3: Adding breadcrumb...");
            sentry::add_breadcrumb(sentry::Breadcrumb {
                message: Some("Test breadcrumb".to_string()),
                category: Some("test".to_string()),
                level: sentry::Level::Info,
                ..Default::default()
            });
            println!("   Breadcrumb added");
            
            // Test 4: Set user context
            println!("👤 Test 4: Setting user context...");
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    id: Some("test-user-123".to_string()),
                    username: Some("test_user".to_string()),
                    email: Some("test@example.com".to_string()),
                    ..Default::default()
                }));
            });
            println!("   User context set");
            
            // Test 5: Custom tag
            println!("🏷️  Test 5: Setting custom tags...");
            sentry::configure_scope(|scope| {
                scope.set_tag("test_run", "simple_sentry_test");
                scope.set_tag("platform", std::env::consts::OS);
            });
            println!("   Custom tags set");
            
            println!("\n✅ All tests completed successfully!");
            println!("🚀 Check your Sentry dashboard to verify events were captured");
            
            // Give some time for events to be sent
            std::thread::sleep(std::time::Duration::from_secs(2));
            
        } else {
            println!("❌ No SENTRY_DSN environment variable found");
            println!("   Set SENTRY_DSN to test Sentry integration");
        }
    }
    
    #[cfg(not(feature = "sentry"))]
    {
        println!("❌ Sentry feature is not enabled");
        println!("   Run with: cargo run --example simple_sentry_test --features=\"sentry\"");
        println!("   And set SENTRY_DSN environment variable to test Sentry integration");
    }
    
    println!("=== Test completed ===");
}
