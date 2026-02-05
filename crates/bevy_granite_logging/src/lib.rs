pub mod buffer;
pub mod config;
pub mod entry;
pub mod macros;
pub mod file;
pub mod output;


pub use buffer::{push_log, LOG_BUFFER};
pub use config::{
    disable_log_category, disable_log_level, disable_log_type, enable_logging, disable_logging,
    setup_logging, LogCategory, LogLevel, LogType, RgbaColor,
};
pub use entry::LogEntry;
pub use output::log;

