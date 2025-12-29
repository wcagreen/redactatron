use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

/// Initialize logging to file and console with proper error handling
/// 
/// This function sets up the logging infrastructure for the Redactatron application.
/// It attempts to create a log file in the platform-specific application data directory,
/// with fallback to the current directory if that fails.
/// 
/// # Logging Configuration
/// - **Format**: Includes millisecond precision timestamps
/// - **Level**: Set to Info by default (can be overridden via `RUST_LOG` environment variable)
/// - **Output**: File-based logging (console fallback if file creation fails)
/// 
/// # Environment Variables
/// - `RUST_LOG`: Override the log level (e.g., `RUST_LOG=debug` for debug output)
/// 
/// # Example Usage
/// In your main.rs:
/// ```ignore
/// fn main() {
///     redactor_core::utils::logging::init();
///     log::info!("Application started");
/// }
/// ```
/// 
/// # Log Levels Used Throughout the Crate
/// - **ERROR**: Critical failures that require attention
/// - **WARN**: Potential issues (currently not extensively used)
/// - **INFO**: Major operations (file loads, conversions started/completed)
/// - **DEBUG**: Detailed operational information (PDFium initialization, dimension calculations, etc.)
/// 
/// # Log File Location
/// - **Windows**: `%APPDATA%/redactatron/logs/redactor.log`
/// - **macOS**: `~/Library/Application Support/redactatron/logs/redactor.log`
/// - **Linux**: `~/.local/share/redactatron/logs/redactor.log`
/// - **Fallback**: `./redactor.log` in current directory
pub fn init() {
    let log_path = get_log_file_path();
    
    // Try to create log directory
    if let Some(parent) = log_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Warning: Could not create log directory {}: {}", parent.display(), e);
        }
    }

    // Configure env_logger to output to file
    let file_result = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path);

    match file_result {
        Ok(file) => {
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Info)
                .format_timestamp_millis()
                .target(env_logger::Target::Pipe(Box::new(file)))
                .init();

            log::info!("=== Redactatron Started ===");
            log::info!("Log file: {}", log_path.display());
        }
        Err(e) => {
            eprintln!("Failed to create log file at {}: {}", log_path.display(), e);
            eprintln!("Logs will not be saved.");
            env_logger::Builder::from_default_env().init();
        }
    }
}

/// Get the path to the log file (tries AppData first, falls back to exe directory)
fn get_log_file_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "redactatron") {
        let log_dir = proj_dirs.data_local_dir().join("logs");
        log_dir.join("redactor.log")
    } else {
        // Fallback to current directory
        PathBuf::from("redactor.log")
    }
}


