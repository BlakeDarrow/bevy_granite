use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};
use serde::Deserialize;
use std::cmp::Ordering;

pub const VERSIONS_TOML: &str = include_str!("../../config/versions.toml");

#[derive(Deserialize, Debug)]
struct FileVersionConfig {
    scene_format: SceneFormatConfig,
}

#[derive(Deserialize, Debug)]
struct SceneFormatConfig {
    current_version: String,
    minimum_supported_version: String,
    next_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    pre_release: Option<String>,
    build: Option<String>,
}

impl Version {
    fn parse(version_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version_str.split('+').collect();
        let (version_part, build) = match parts.len() {
            1 => (parts[0], None),
            2 => (parts[0], Some(parts[1].to_string())),
            _ => return Err("Invalid version format: too many '+' separators".to_string()),
        };

        let parts: Vec<&str> = version_part.split('-').collect();
        let (core_version, pre_release) = match parts.len() {
            1 => (parts[0], None),
            _ => {
                let pre = parts[1..].join("-");
                (parts[0], if pre.is_empty() { None } else { Some(pre) })
            }
        };

        let version_numbers: Vec<&str> = core_version.split('.').collect();
        if version_numbers.len() != 3 {
            return Err("Version must have exactly 3 numbers (major.minor.patch)".to_string());
        }

        let major = version_numbers[0]
            .parse::<u32>()
            .map_err(|_| "Invalid major version number")?;
        let minor = version_numbers[1]
            .parse::<u32>()
            .map_err(|_| "Invalid minor version number")?;
        let patch = version_numbers[2]
            .parse::<u32>()
            .map_err(|_| "Invalid patch version number")?;

        Ok(Version {
            major,
            minor,
            patch,
            pre_release,
            build,
        })
    }

    fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major.minor.patch first
        match (
            self.major.cmp(&other.major),
            self.minor.cmp(&other.minor),
            self.patch.cmp(&other.patch),
        ) {
            (Ordering::Equal, Ordering::Equal, Ordering::Equal) => {
                // Core versions are equal, now compare pre-release
                match (&self.pre_release, &other.pre_release) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Greater, // Stable > pre-release
                    (Some(_), None) => Ordering::Less,    // Pre-release < stable
                    (Some(a), Some(b)) => {
                        // Both are pre-releases, compare lexicographically
                        // This is a simplified comparison
                        a.cmp(b)
                    }
                }
            }
            (Ordering::Equal, Ordering::Equal, patch_cmp) => patch_cmp,
            (Ordering::Equal, minor_cmp, _) => minor_cmp,
            (major_cmp, _, _) => major_cmp,
        }
    }
}

/// Check if the given version is compatible with the current format
pub fn is_scene_version_compatible(version: &str) -> bool {
    let current_version_str = get_current_scene_version();
    let min_version_str = get_minimum_scene_version();

    // Parse versions
    let version = match Version::parse(version) {
        Ok(v) => v,
        Err(e) => {
            log!(
                LogType::Game,
                LogLevel::Error,
                LogCategory::System,
                "Failed to parse version '{}': {}",
                version,
                e
            );
            return false;
        }
    };

    let current_version = match Version::parse(&current_version_str) {
        Ok(v) => v,
        Err(e) => {
            log!(
                LogType::Game,
                LogLevel::Error,
                LogCategory::System,
                "Failed to parse current version '{}': {}",
                current_version_str,
                e
            );
            return false;
        }
    };

    let min_version = match Version::parse(&min_version_str) {
        Ok(v) => v,
        Err(e) => {
            log!(
                LogType::Game,
                LogLevel::Error,
                LogCategory::System,
                "Failed to parse minimum version '{}': {}",
                min_version_str,
                e
            );
            return false;
        }
    };

    // Check if version matches current exactly
    if version == current_version {
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "Version '{}' matches current version exactly",
            version.to_string()
        );
        return true;
    }

    // Check if version is at least the minimum supported
    if version >= min_version {
        if version < current_version {
            let version_type = if version.is_pre_release() {
                "pre-release"
            } else {
                "stable"
            };
            log!(
                LogType::Game,
                LogLevel::Info,
                LogCategory::System,
                "Loading older compatible {} version '{}' (current: '{}', min supported: '{}')",
                version_type,
                version.to_string(),
                current_version.to_string(),
                min_version.to_string()
            );
        } else {
            // version > current_version
            let version_type = if version.is_pre_release() {
                "pre-release"
            } else {
                "stable"
            };
            log!(
                LogType::Game,
                LogLevel::Warning,
                LogCategory::System,
                "Loading newer {} version '{}' than current '{}' - this may cause issues",
                version_type,
                version.to_string(),
                current_version.to_string()
            );
        }
        return true;
    }

    // Version is below minimum supported
    let version_type = if version.is_pre_release() {
        "pre-release"
    } else {
        "stable"
    };
    log!(
        LogType::Game,
        LogLevel::Error,
        LogCategory::System,
        "Version '{}' ({}) is below minimum supported version '{}'. Current version is '{}'.",
        version.to_string(),
        version_type,
        min_version.to_string(),
        current_version.to_string()
    );
    false
}

// For Scene Metadata
pub fn get_current_scene_version() -> String {
    match toml::from_str::<FileVersionConfig>(VERSIONS_TOML) {
        Ok(config) => config.scene_format.current_version,
        Err(_) => "1.0.0".to_string(),
    }
}
// For Scene Metadata
pub fn get_beta_scene_version() -> String {
    match toml::from_str::<FileVersionConfig>(VERSIONS_TOML) {
        Ok(config) => config.scene_format.next_version,
        Err(_) => "1.0.0".to_string(),
    }
}

// For Scene Metadata
pub fn get_minimum_scene_version() -> String {
    match toml::from_str::<FileVersionConfig>(VERSIONS_TOML) {
        Ok(min) => min.scene_format.minimum_supported_version,
        Err(_) => "1.0.0".to_string(),
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre_release) = self.pre_release {
            write!(f, "-{}", pre_release)?;
        }
        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}
