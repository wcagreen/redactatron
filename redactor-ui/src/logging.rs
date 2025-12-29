use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

/// Initialize logging to file
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
        .append(true)
        .open(&log_path);

    match file_result {
        Ok(file) => {
            env_logger::Builder::from_default_env()
                .format_timestamp_millis()
                .target(env_logger::Target::Pipe(Box::new(file)))
                .init();

            log::info!("=== Rusty Redactor Started ===");
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


