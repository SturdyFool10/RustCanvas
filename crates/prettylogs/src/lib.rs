//! Pretty logs for RustCanvas.

use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the tracing subscriber with custom filtering rules.
///
/// This function sets up logging with the following rules:
/// - All crates from the local project log at the TRACE level
/// - External crates only log at the WARN level or above
///
/// # Example
/// ```
/// prettylogs::init_logging();
/// tracing::info!("This log from your code will be visible");
/// ```
pub fn init_logging() {
    // Create an environment filter that:
    // 1. Sets external crates to only log at WARN level or higher (always)
    // 2. Sets our internal crates to log at appropriate level based on build profile:
    //    - In debug: TRACE level
    //    - In release: INFO level (skip debug and trace)

    // Determine minimum log level based on build configuration
    #[cfg(debug_assertions)]
    let internal_level = "trace";
    #[cfg(not(debug_assertions))]
    let internal_level = "info";

    // Build the filter directive string
    let filter_directive = format!(
        "rustcanvas={0},appstate={0},authentication={0},config={0},db={0},macros={0},prettylogs={0},utils={0},webserver={0},warn",
        internal_level
    );

    let filter = EnvFilter::builder()
        // Add any specific crates from our project here to enable appropriate logging
        .parse(&filter_directive)
        .expect("Invalid filter directive");

    // Initialize the tracing subscriber with the filter and no time/date
    tracing_subscriber::registry()
        .with(
            fmt::layer().with_target(true).without_time(), // Remove timestamp from output
        )
        .with(filter)
        .init();

    #[cfg(debug_assertions)]
    tracing::debug!("Logging initialized with trace level enabled");
    #[cfg(not(debug_assertions))]
    tracing::info!("Logging initialized (debug disabled in release mode)");
}

/// Initialize the tracing subscriber with a custom filter string.
///
/// This function allows for more fine-grained control over logging levels
/// by accepting a custom filter string in the format expected by tracing's EnvFilter.
///
/// # Parameters
///
/// * `filter_str` - A custom filter directive string
///
/// # Example
/// ```
/// // Enable debug for our code, info for some_dependency, and warn for everything else
/// prettylogs::init_logging_with_filter("rustcanvas=debug,some_dependency=info,warn");
/// ```
pub fn init_logging_with_filter(filter_str: &str) {
    // In release mode, we'll respect the provided filter but ensure debug logs are disabled
    // for any crates that don't explicitly override this
    #[cfg(not(debug_assertions))]
    let filter_str = if !filter_str.contains("debug=") && !filter_str.contains("=debug") {
        format!("{},debug=off", filter_str)
    } else {
        filter_str.to_string()
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::try_new(filter_str).expect("Invalid filter directive"));

    tracing_subscriber::registry()
        .with(
            fmt::layer().with_target(true).without_time(), // Remove timestamp from output
        )
        .with(filter)
        .init();

    #[cfg(debug_assertions)]
    tracing::debug!("Logging initialized with custom filter: {}", filter_str);
    #[cfg(not(debug_assertions))]
    tracing::info!("Logging initialized with custom filter (debug disabled in release mode)");
}
