use crate::buffer::push_log;
use crate::config::{
    LogCategory, LogLevel, LogType, ENABLED_LOG_CATEGORIES, ENABLED_LOG_LEVELS, ENABLED_LOG_TYPES,
};
use crate::entry::LogEntry;
use crate::file::write_to_file;
use chrono::Local;
use colored::*;
use textwrap::wrap;

// std out line width
const MAX_LINE_WIDTH: usize = 78;

// std out color
fn get_colored_level(level: LogLevel) -> ColoredString {
    match level {
        LogLevel::Info => " ".white(),
        LogLevel::OK => "(OK) ".green(),
        LogLevel::Warning => "(WARNING) ".yellow(),
        LogLevel::Error => "(ERROR) ".red().underline(),
        LogLevel::Critical => "(CRITICAL) ".red().underline(),
    }
}

// std out color
fn get_colored_message(level: LogLevel, message: &str) -> ColoredString {
    match level {
        LogLevel::Info => message.white(),
        LogLevel::OK => message.green(),
        LogLevel::Warning => message.yellow(),
        LogLevel::Error => message.red(),
        LogLevel::Critical => message.red().bold().on_black().underline(),
    }
}

//std out color
fn get_colored_type(r#type: LogType) -> ColoredString {
    match r#type {
        LogType::Game => "GAME ".dimmed(),
        LogType::Editor => "EDTR ".dimmed(),
    }
}

// std out color
fn get_colored_category(category: LogCategory) -> ColoredString {
    match category {
        LogCategory::Entity => "[ENTITY]".purple().bold(),
        LogCategory::Debug => "".dimmed(),
        LogCategory::Asset => "[ASSET]".yellow(),
        LogCategory::UI => "[UI]".green(),
        LogCategory::Input => "[INPUT]".blue(),
        LogCategory::System => "[SYSTEM]".dimmed(),
        LogCategory::Network => "[NETWORK]".blue(),
        LogCategory::Other => "[OTHER]".red(),
        LogCategory::Blank => "".white(),
        LogCategory::Serialization => "[SERDE]".truecolor(200, 62, 22),
    }
}

// Handles both buffer and stdout
pub fn log(r#type: LogType, level: LogLevel, category: LogCategory, message: String) {
    let timestamp = Local::now().format("%m/%d/%Y-%H:%M:%S ").to_string();
    let type_prefix = get_colored_type(r#type);
    let level_prefix = get_colored_level(level);
    let category_prefix = get_colored_category(category);
    let wrapped_message = wrap(&message, MAX_LINE_WIDTH);

    let entry = LogEntry {
        timestamp: timestamp.clone(),
        log_type: r#type,
        level,
        category,
        message: message.clone(),
    };

    // add to buffer for things like UI
    push_log(entry.clone());
    if let Err(e) = write_to_file(&entry) {
        eprintln!("Failed to write to log file: {}", e);
    }
    // stdout of log entry
    let cat_set = ENABLED_LOG_CATEGORIES.lock().unwrap();
    let level_set = ENABLED_LOG_LEVELS.lock().unwrap();
    let type_set = ENABLED_LOG_TYPES.lock().unwrap();
    if cat_set.contains(&category) && level_set.contains(&level) && type_set.contains(&r#type) {
        if let Some((first, rest)) = wrapped_message.split_first() {
            let colored_first_line = get_colored_message(level, first);
            println!(
                "{}{}{}{}{}",
                timestamp.dimmed(),
                type_prefix,
                category_prefix,
                level_prefix,
                colored_first_line
            );

            for line in rest {
                let colored_line = get_colored_message(level, &format!("\t{}", line));
                println!("{}", colored_line);
            }
        }
    }
}
