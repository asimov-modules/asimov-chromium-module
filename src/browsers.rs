use miette::{IntoDiagnostic, Result, WrapErr, miette};
use phf::phf_map;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::string::{String, ToString};
use std::vec::Vec;
use std::{eprintln, format, vec};

use crate::specialized;

/// Configuration for browser-specific user data paths.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct UserDataPath {
    url_prefix: &'static str,
    linux: &'static str,
    macos: &'static str,
    windows: &'static str,
}

/// Browser configuration and operations.
pub struct BrowserConfig {
    name: &'static str,
    paths: &'static UserDataPath,
}

/// Supported browsers enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    Chrome,
    Brave,
    Edge,
    Chromium,
    Arc,
    Opera,
}

impl BrowserConfig {
    pub fn name(&self) -> &str {
        self.name
    }

    fn browser_type(&self) -> Option<Browser> {
        match self.name {
            "chrome" => Some(Browser::Chrome),
            "brave" => Some(Browser::Brave),
            "edge" => Some(Browser::Edge),
            "chromium" => Some(Browser::Chromium),
            "arc" => Some(Browser::Arc),
            "opera" => Some(Browser::Opera),
            _ => None,
        }
    }

    pub fn profile_path(&self, profile_name: Option<&str>) -> Result<PathBuf> {
        let mut path = self.platform_user_data_path()?;
        path.push(profile_name.unwrap_or("Default"));
        if !path.is_dir() {
            return Err(miette!(
                "Profile path not found for browser '{}': {}",
                self.name,
                path.display()
            ));
        }
        Ok(path)
    }

    fn platform_user_data_path(&self) -> Result<PathBuf> {
        let mut path = PathBuf::new();

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME")
                .into_diagnostic()
                .wrap_err("HOME environment variable must be set")?;
            path.push(home);
            path.push(self.paths.linux);
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .into_diagnostic()
                .wrap_err("HOME environment variable must be set")?;
            path.push(home);
            path.push(self.paths.macos);
        }

        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA")
                .into_diagnostic()
                .wrap_err("LOCALAPPDATA environment variable must be set")?;
            path.push(local_app_data);
            path.push(self.paths.windows);
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            return Err(miette!("Unsupported operating system"));
        }

        Ok(path)
    }

    pub fn bookmarks_path(&self, profile_name: Option<&str>) -> Result<PathBuf> {
        match self.browser_type() {
            Some(Browser::Arc) => self
                .platform_user_data_path()
                .map(|path| path.join("StorableSidebar.json")),
            _ => self
                .profile_path(profile_name)
                .map(|path| path.join("Bookmarks")),
        }
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        match self.browser_type() {
            Some(Browser::Arc) => {
                let mut base_path = self.platform_user_data_path()?;
                base_path.push("User Data");
                let mut profiles = Vec::new();

                if base_path.is_dir() {
                    for entry in std::fs::read_dir(&base_path).into_diagnostic()? {
                        let entry = entry.into_diagnostic()?;
                        if entry.file_type().into_diagnostic()?.is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                if matches!(name, "Default") || name.starts_with("Profile ") {
                                    profiles.push(name.to_string());
                                }
                            }
                        }
                    }
                }

                if profiles.is_empty() {
                    profiles.push("Default".to_string());
                }

                Ok(profiles)
            },
            _ => {
                let profile_path = self.profile_path(None)?;
                let base_path = profile_path
                    .parent()
                    .ok_or_else(|| miette!("Failed to get parent directory"))?;

                let mut profiles = Vec::new();
                for entry in std::fs::read_dir(base_path).into_diagnostic()? {
                    let entry = entry.into_diagnostic()?;
                    if entry.file_type().into_diagnostic()?.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            let profile_dir = base_path.join(name);
                            let bookmarks_file = profile_dir.join("Bookmarks");

                            if matches!(name, "Default")
                                || name.starts_with("Profile ")
                                || name.starts_with("Profile")
                                || bookmarks_file.is_file()
                            {
                                profiles.push(name.to_string());
                            }
                        }
                    }
                }

                if profiles.is_empty() {
                    profiles.push("Default".to_string());
                }

                Ok(profiles)
            },
        }
    }
}

/// Converts Arc bookmarks to standard Chromium format
fn convert_arc_to_bookmarks(arc_data: Value, profile: Option<&str>) -> Result<Value> {
    specialized::arc::convert_arc_bookmarks_to_chromium(arc_data, profile)
}

/// Converts Arc bookmarks to standard Chromium format using numeric profile index
fn convert_arc_to_bookmarks_numeric(arc_data: Value, profile_index: Option<u32>) -> Result<Value> {
    specialized::arc::convert_arc_bookmarks_to_chromium_numeric(arc_data, profile_index)
}

// Use shared implementation from utils
use crate::utils::map_numeric_profile_to_name;

static SUPPORTED_BROWSERS: phf::Map<&'static str, UserDataPath> = phf_map! {
    "chrome" => UserDataPath {
        url_prefix: "chrome://bookmarks",
        linux: ".config/google-chrome",
        macos: "Library/Application Support/Google/Chrome",
        windows: "Google/Chrome/User Data",
    },
    "brave" => UserDataPath {
        url_prefix: "brave://bookmarks",
        linux: ".config/BraveSoftware/Brave-Browser",
        macos: "Library/Application Support/BraveSoftware/Brave-Browser",
        windows: "BraveSoftware/Brave-Browser/User Data",
    },
    "edge" => UserDataPath {
        url_prefix: "edge://bookmarks",
        linux: ".config/microsoft-edge",
        macos: "Library/Application Support/Microsoft Edge",
        windows: "Microsoft/Edge/User Data",
    },
    "chromium" => UserDataPath {
        url_prefix: "chromium://bookmarks",
        linux: ".config/chromium",
        macos: "Library/Application Support/Chromium",
        windows: "Chromium/User Data",
    },
    "arc" => UserDataPath {
        url_prefix: "arc://bookmarks",
        linux: ".config/Arc",
        macos: "Library/Application Support/Arc",
        windows: "Arc/User Data",
    },
    "opera" => UserDataPath {
        url_prefix: "opera://bookmarks",
        linux: ".config/opera",
        macos: "Library/Application Support/com.operasoftware.Opera",
        windows: "Opera/User Data",
    },
};

pub fn get_browser_from_url(url: &str) -> Option<BrowserConfig> {
    SUPPORTED_BROWSERS
        .entries()
        .find(|(_, config)| {
            url == config.url_prefix || url.starts_with(&format!("{}/", config.url_prefix))
        })
        .map(|(name, paths)| BrowserConfig { name, paths })
}

pub fn fetch_bookmarks(url: &str) -> Result<Vec<Value>> {
    let browser = get_browser_from_url(url).ok_or_else(|| {
        miette!(
            "Unsupported URL: {}. Supported prefixes: {:?}",
            url,
            SUPPORTED_BROWSERS
                .entries()
                .map(|(_, config)| config.url_prefix)
                .collect::<Vec<_>>()
        )
    })?;

    if browser.browser_type() == Some(Browser::Arc) {
        let profile_index: Option<u32> = if url.starts_with("arc://bookmarks/") {
            let profile_part = url.strip_prefix("arc://bookmarks/").unwrap_or("");
            if profile_part.is_empty() {
                None
            } else {
                if profile_part == "Default" {
                    Some(1)
                } else {
                    profile_part.parse::<u32>().ok()
                }
            }
        } else {
            None
        };

        if let Ok(path) = browser.bookmarks_path(None) {
            if let Ok(bookmarks) = read_bookmarks_file_with_numeric_profile(&path, profile_index) {
                return Ok(vec![bookmarks]);
            }
        }

        return Err(miette!(
            "No valid bookmarks files found for browser: {}",
            browser.name()
        ));
    }

    let profile_index: Option<u32> = if url.starts_with(&format!("{}/", browser.paths.url_prefix)) {
        let profile_part = url
            .strip_prefix(&format!("{}/", browser.paths.url_prefix))
            .unwrap_or("");
        if profile_part.is_empty() {
            None
        } else {
            profile_part.parse::<u32>().ok()
        }
    } else if url == browser.paths.url_prefix {
        None
    } else {
        None
    };

    let available_profiles = browser.list_profiles()?;

    if available_profiles.is_empty() {
        return Err(miette!("No profiles found for browser: {}", browser.name()));
    }

    let mut outputs = Vec::new();
    match profile_index {
        Some(0) | None => {
            let transform = crate::BookmarksTransform::new()
                .map_err(|e| miette!("Failed to create BookmarksTransform: {}", e))?;

            let mut all_bookmarks = Vec::new();

            for profile in &available_profiles {
                if let Ok(path) = browser.bookmarks_path(Some(profile)) {
                    if let Ok(bookmarks) = read_bookmarks_file(&path, Some(profile)) {
                        all_bookmarks.push(bookmarks);
                    }
                }
            }

            let merged_result = transform
                .execute_multiple(all_bookmarks)
                .map_err(|e| miette!("Failed to transform and merge bookmarks: {}", e))?;

            outputs.push(merged_result);
        },
        Some(index) => {
            eprintln!("DEBUG: Profile index requested: {}", index);
            if let Some(profile_name) = map_numeric_profile_to_name(&available_profiles, index) {
                eprintln!("DEBUG: Mapped to profile name: {}", profile_name);
                if let Ok(path) = browser.bookmarks_path(Some(&profile_name)) {
                    eprintln!("DEBUG: Reading bookmarks from path: {}", path.display());
                    if let Ok(bookmarks) = read_bookmarks_file(&path, Some(&profile_name)) {
                        eprintln!(
                            "DEBUG: Successfully read bookmarks for profile: {}",
                            profile_name
                        );
                        outputs.push(bookmarks);
                    }
                }
            } else {
                return Err(miette!(
                    "Profile index {} not found. Available profiles: {}",
                    index,
                    available_profiles.join(", ")
                ));
            }
        },
    }

    if outputs.is_empty() {
        return Err(miette!(
            "No valid bookmarks files found for browser: {}",
            browser.name()
        ));
    }

    Ok(outputs)
}

fn read_bookmarks_file(path: &Path, profile: Option<&str>) -> Result<Value> {
    if !path.is_file() {
        return Err(miette!("Bookmarks file not found at {}", path.display()));
    }

    let input = std::fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read bookmarks at {}", path.display()))?;

    if path.file_name().and_then(|n| n.to_str()) == Some("StorableSidebar.json") {
        let arc_data: Value = serde_json::from_str(&input)
            .into_diagnostic()
            .wrap_err_with(|| {
                format!(
                    "Failed to parse Arc StorableSidebar.json from {}",
                    path.display()
                )
            })?;

        convert_arc_to_bookmarks(arc_data, profile)
    } else {
        serde_json::from_str(&input)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to parse bookmarks JSON from {}", path.display()))
    }
}

fn read_bookmarks_file_with_numeric_profile(
    path: &Path,
    profile_index: Option<u32>,
) -> Result<Value> {
    if !path.is_file() {
        return Err(miette!("Bookmarks file not found at {}", path.display()));
    }

    let input = std::fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read bookmarks at {}", path.display()))?;

    if path.file_name().and_then(|n| n.to_str()) == Some("StorableSidebar.json") {
        let arc_data: Value = serde_json::from_str(&input)
            .into_diagnostic()
            .wrap_err_with(|| {
                format!(
                    "Failed to parse Arc StorableSidebar.json from {}",
                    path.display()
                )
            })?;

        convert_arc_to_bookmarks_numeric(arc_data, profile_index)
    } else {
        serde_json::from_str(&input)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to parse bookmarks JSON from {}", path.display()))
    }
}
