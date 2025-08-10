use miette::{IntoDiagnostic, Result, WrapErr, miette};
use phf::phf_map;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::string::{String, ToString};
use std::vec::Vec;
use std::{boxed::Box, format, print, vec};

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
            Some(Browser::Arc) => {
                // Arc sempre usa o StorableSidebar.json do diretório principal
                // mas o profile_name é passado para convert_arc_to_bookmarks
                self.platform_user_data_path()
                    .map(|path| path.join("StorableSidebar.json"))
            },
            _ => self
                .profile_path(profile_name)
                .map(|path| path.join("Bookmarks")),
        }
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        match self.browser_type() {
            Some(Browser::Arc) => {
                // Arc stores profiles in the User Data subdirectory
                let mut base_path = self.platform_user_data_path()?;
                base_path.push("User Data");
                let mut profiles = Vec::new();

                // Check if the User Data path exists
                if base_path.is_dir() {
                    for entry in std::fs::read_dir(&base_path).into_diagnostic()? {
                        let entry = entry.into_diagnostic()?;
                        if entry.file_type().into_diagnostic()?.is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                // Arc profiles are typically named like "Profile 1", "Profile 2", etc.
                                // or "Default" for the default profile
                                if matches!(name, "Default") || name.starts_with("Profile ") {
                                    profiles.push(name.to_string());
                                }
                            }
                        }
                    }
                }

                // If no profiles found, return at least "Default"
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
                            if matches!(name, "Default") || name.starts_with("Profile ") {
                                profiles.push(name.to_string());
                            }
                        }
                    }
                }
                Ok(profiles)
            },
        }
    }
}

/// Converts Arc's StorableSidebar.json format to standard Chromium bookmarks format
fn convert_arc_to_bookmarks(arc_data: Value, profile: Option<&str>) -> Result<Value> {
    specialized::arc::convert_arc_bookmarks_to_chromium(arc_data, profile)
}

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

    // Arc browser special handling
    if browser.browser_type() == Some(Browser::Arc) {
        let profile: Option<String> = if url.starts_with("arc://bookmarks/") {
            let profile_part = url.strip_prefix("arc://bookmarks/").unwrap_or("");
            print!("OLHA profile_part: {}", profile_part);
            if profile_part.is_empty() {
                None
            } else {
                let decoded_profile = profile_part
                    .replace("%20", " ")
                    .replace("\\ ", " ")
                    .replace("+", " ");

                Some(decoded_profile)
            }
        } else {
            url.strip_prefix(browser.paths.url_prefix)
                .and_then(|suffix| suffix.strip_prefix('/').filter(|s| !s.is_empty()))
                .map(|profile| profile.to_string())
        };
        print!("OLHA profile novamente: {:?}", profile);

        let profile_to_use = if let Some(profile_name) = profile.as_deref() {
            if profile_name == "Default" {
                "Default"
            } else if profile_name.starts_with("Profile") && !profile_name.contains(" ") {
                let number = profile_name.strip_prefix("Profile").unwrap_or("");
                if !number.is_empty() {
                    let formatted_profile = format!("Profile {}", number);
                    let profile_string = Box::leak(Box::new(formatted_profile));
                    profile_string.as_str()
                } else {
                    profile_name
                }
            } else {
                profile_name
            }
        } else {
            "Default"
        };

        if let Ok(path) = browser.bookmarks_path(Some(profile_to_use)) {
            if let Ok(bookmarks) = read_bookmarks_file(&path, Some(profile_to_use)) {
                return Ok(vec![bookmarks]);
            }
        }

        return Err(miette!(
            "No valid bookmarks files found for browser: {}",
            browser.name()
        ));
    }

    // Other browsers
    let profiles: Vec<String> = url
        .strip_prefix(browser.paths.url_prefix)
        .and_then(|suffix| suffix.strip_prefix('/').filter(|s| !s.is_empty()))
        .map(|profile| vec![profile.to_string()])
        .unwrap_or_else(|| browser.list_profiles().unwrap_or_default());

    if profiles.is_empty() {
        return Err(miette!("No profiles found for browser: {}", browser.name()));
    }

    let mut outputs = Vec::new();

    for profile in profiles {
        if let Ok(path) = browser.profile_path(Some(&profile)) {
            if let Ok(bookmarks) = read_bookmarks_file(&path, Some(&profile)) {
                outputs.push(bookmarks);
            }
        }
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

    // Check if this is an Arc StorableSidebar.json file
    if path.file_name().and_then(|n| n.to_str()) == Some("StorableSidebar.json") {
        // Parse as Arc format and convert to standard bookmarks format
        let arc_data: Value = serde_json::from_str(&input)
            .into_diagnostic()
            .wrap_err_with(|| {
                format!(
                    "Failed to parse Arc StorableSidebar.json from {}",
                    path.display()
                )
            })?;

        // Convert Arc format to standard bookmarks format
        convert_arc_to_bookmarks(arc_data, profile)
    } else {
        // Standard Chromium bookmarks format
        serde_json::from_str(&input)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to parse bookmarks JSON from {}", path.display()))
    }
}
