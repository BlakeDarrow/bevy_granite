use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::log;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LogCategory {
    Entity,
    Asset,
    UI,
    Input,
    System,
    Network,
    Other,
    Debug,
    Blank,
    Serialization,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum LogLevel {
    Info,
    OK,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LogType {
    Editor,
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor(pub u8, pub u8, pub u8, pub u8);

impl RgbaColor {
    pub const WHITE: Self = Self(240, 240, 240, 255);
    pub const BLACK: Self = Self(20, 20, 20, 255);
    pub const RED: Self = Self(200, 50, 50, 255);
    pub const GREEN: Self = Self(50, 200, 50, 255);
    pub const BLUE: Self = Self(50, 50, 200, 255);
    pub const YELLOW: Self = Self(220, 220, 50, 255);
    pub const CYAN: Self = Self(50, 200, 200, 255);
    pub const MAGENTA: Self = Self(200, 50, 200, 255);
    pub const GRAY: Self = Self(100, 100, 100, 255);
}

impl LogType {
    pub fn all() -> Vec<Self> {
        vec![Self::Game, Self::Editor]
    }
}

impl LogCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Entity,
            Self::Asset,
            Self::UI,
            Self::Input,
            Self::System,
            Self::Network,
            Self::Other,
            Self::Debug,
            Self::Blank,
        ]
    }

    pub fn ui_color(&self) -> RgbaColor {
        match self {
            LogCategory::Entity => RgbaColor(180, 0, 255, 255),
            LogCategory::Asset => RgbaColor(255, 193, 7, 255),
            LogCategory::UI => RgbaColor::GREEN,
            LogCategory::Input => RgbaColor::BLUE,
            LogCategory::System => RgbaColor::GRAY,
            LogCategory::Network => RgbaColor::BLUE,
            LogCategory::Other => RgbaColor(255, 152, 0, 255),
            LogCategory::Blank => RgbaColor::WHITE,
            LogCategory::Debug => RgbaColor::GRAY,
            LogCategory::Serialization => RgbaColor(200, 62, 22, 255),
        }
    }
}

impl LogLevel {
    pub fn all() -> Vec<Self> {
        vec![
            Self::OK,
            Self::Warning,
            Self::Error,
            Self::Critical,
            Self::Info,
        ]
    }
    pub fn info() -> Vec<Self> {
        vec![Self::Info]
    }
    pub fn minimal() -> Vec<Self> {
        vec![Self::OK, Self::Warning, Self::Error, Self::Critical]
    }

    pub fn errors() -> Vec<Self> {
        vec![Self::Warning, Self::Error, Self::Critical]
    }

    pub fn ui_color(&self) -> RgbaColor {
        match self {
            LogLevel::Info => RgbaColor::GRAY,
            LogLevel::OK => RgbaColor::GREEN,
            LogLevel::Warning => RgbaColor::YELLOW,
            LogLevel::Error => RgbaColor::RED,
            LogLevel::Critical => RgbaColor(255, 0, 0, 255),
        }
    }
}

// std out config, irrelevant for buffer
// -----------------------------------------------------------------------------------------------------------------------

lazy_static! {
    pub static ref ENABLED_LOG_CATEGORIES: Mutex<HashSet<LogCategory>> = Mutex::new(HashSet::new());
}

lazy_static! {
    pub static ref ENABLED_LOG_LEVELS: Mutex<HashSet<LogLevel>> = Mutex::new(HashSet::new());
}

lazy_static! {
    pub static ref ENABLED_LOG_TYPES: Mutex<HashSet<LogType>> = Mutex::new(HashSet::new());
}

pub fn disable_log_category(category: LogCategory) {
    let mut set = ENABLED_LOG_CATEGORIES.lock().unwrap();
    set.remove(&category);
}

pub fn disable_log_level(level: LogLevel) {
    let mut set = ENABLED_LOG_LEVELS.lock().unwrap();
    set.remove(&level);
}

pub fn disable_log_type(r#type: LogType) {
    let mut set = ENABLED_LOG_TYPES.lock().unwrap();
    set.remove(&r#type);
}

pub fn setup_logging() {
    let categories = LogCategory::all();
    let levels = LogLevel::all();
    let types = LogType::all();

    {
        let mut category_set = ENABLED_LOG_CATEGORIES.lock().unwrap();
        for category in categories.iter() {
            category_set.insert(*category);
        }
    }

    {
        let mut level_set = ENABLED_LOG_LEVELS.lock().unwrap();
        for level in levels.iter() {
            level_set.insert(*level);
        }
    }

    {
        let mut types_set = ENABLED_LOG_TYPES.lock().unwrap();
        for r#type in types.iter() {
            types_set.insert(*r#type);
        }
    }
    log!(
        LogType::Game,
        LogLevel::Info,
        LogCategory::Blank,
        "--------------------"
    );
    log!(
        LogType::Game,
        LogLevel::Info,
        LogCategory::System,
        "Logging initialized and setup, this is a new session."
    );
    log!(
        LogType::Game,
        LogLevel::Info,
        LogCategory::Blank,
        "--------------------"
    );
}
// -----------------------------------------------------------------------------------------------------------------------
