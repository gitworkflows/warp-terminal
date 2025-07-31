use iced::Application;
use warp_terminal::WarpTerminal;
use warp_terminal::monitoring::{init_monitoring, monitor_startup, monitor_shutdown, report_critical_error};
use std::process;

fn main() -> iced::Result {
    // Initialize monitoring systems (Sentry, tracing)
    if let Err(e) = init_monitoring() {
        eprintln!("Failed to initialize monitoring: {}", e);
        // Continue execution even if monitoring fails
    }
    
    // Monitor application startup
    monitor_startup();
    
    // Set up panic hook to report to Sentry
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let panic_message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic occurred".to_string()
        };
        
        let location = if let Some(location) = panic_info.location() {
            format!(" at {}:{}:{}", location.file(), location.line(), location.column())
        } else {
            String::new()
        };
        
        let error = anyhow::anyhow!("Panic: {}{}", panic_message, location);
        report_critical_error(&error, "Application panic");
        
        // Call the default panic handler
        default_panic(panic_info);
    }));
    
    // Set up graceful shutdown handler
    let result = WarpTerminal::run(iced::Settings::default());
    
    // Monitor application shutdown
    monitor_shutdown();
    
    // Flush Sentry events before exit
    if let Some(sentry) = warp_terminal::monitoring::sentry() {
        if sentry.is_enabled() {
            // Give Sentry time to send any pending events
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    
    result
}
