// This is free and unencumbered software released into the public domain.

use phf::phf_map;
use serde_json::Value;
use std::boxed::Box;
use std::path::PathBuf;
use std::string::{String, ToString};
use std::vec::Vec;
use std::{format, io};

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

impl BrowserConfig {
    /// Returns the browser name.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Constructs the profile path for the given profile name (defaults to "Default").
    pub fn profile_path(&self, profile_name: Option<&str>) -> io::Result<PathBuf> {
        let mut path = PathBuf::new();

        #[cfg(unix)]
        {
            let home = std::env::var("HOME").map_err(|_| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "HOME environment variable must be set",
                )
            })?;
            path.push(home);
            #[cfg(target_os = "linux")]
            path.push(self.paths.linux);
            #[cfg(target_os = "macos")]
            path.push(self.paths.macos);
            #[cfg(not(any(target_os = "linux", target_os = "macos")))]
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Unsupported Unix platform",
            ));
        }

        #[cfg(not(unix))]
        {
            let local_app_data = std::env::var("LOCALAPPDATA").map_err(|_| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "LOCALAPPDATA environment variable must be set",
                )
            })?;
            path.push(local_app_data);
            path.push(self.paths.windows);
        }

        path.push(profile_name.unwrap_or("Default"));
        if !path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Profile path not found: {}", path.display()),
            ));
        }
        Ok(path)
    }

    /// Constructs the bookmarks file path for the given profile.
    pub fn bookmarks_path(&self, profile_name: Option<&str>) -> io::Result<PathBuf> {
        self.profile_path(profile_name)
            .map(|path| path.join("Bookmarks"))
    }

    /// Lists all profile directories under the browser's user data path.
    pub fn list_profiles(&self) -> io::Result<Vec<String>> {
        let base_path = self
            .profile_path(None)?
            .parent()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Failed to get parent directory")
            })?
            .to_path_buf();
        let mut profiles = Vec::new();

        for entry in std::fs::read_dir(base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Include only directories that are likely profiles
                    if name == "Default" || name.starts_with("Profile ") {
                        profiles.push(name.to_string());
                    }
                }
            }
        }

        Ok(profiles)
    }
}

// Static map of supported browsers and their user data paths.
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
};

/// Finds the browser configuration based on the URL prefix.
pub fn get_browser_from_url(url: &str) -> Option<BrowserConfig> {
    SUPPORTED_BROWSERS
        .into_iter()
        .find(|(_, config)| url.starts_with(config.url_prefix))
        .map(|(name, paths)| BrowserConfig { name, paths })
}

/// Fetches bookmarks data for the given URL, handling all profiles or a specific one.
/// Returns a vector of JSON values for each valid bookmarks file found.
pub fn fetch_bookmarks(url: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let browser = get_browser_from_url(url).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Unsupported URL: {}. Supported prefixes: {:?}",
                url,
                SUPPORTED_BROWSERS
                    .into_iter()
                    .map(|(_, config)| config.url_prefix)
                    .collect::<Vec<_>>()
            ),
        )
    })?;

    let profile_name = url
        .strip_prefix(browser.paths.url_prefix)
        .and_then(|suffix| suffix.strip_prefix("/").filter(|s| !s.is_empty()));

    let mut outputs = Vec::new();

    if let Some(profile) = profile_name {
        // Fetch bookmarks for a specific profile
        let path = browser.bookmarks_path(Some(profile))?;
        if path.is_file() {
            let input = std::fs::read_to_string(&path)?;
            outputs.push(serde_json::from_str(&input)?);
        } else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Bookmarks file not found for profile: {} in browser: {}",
                    profile,
                    browser.name()
                ),
            )));
        }
    } else {
        // Fetch bookmarks from all valid profiles
        let profiles = browser.list_profiles()?;
        for profile in profiles {
            if let Ok(path) = browser.bookmarks_path(Some(&profile)) {
                if path.is_file() {
                    let input = std::fs::read_to_string(&path)?;
                    outputs.push(serde_json::from_str(&input)?);
                }
            }
        }

        if outputs.is_empty() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "No valid bookmarks files found for browser: {}",
                    browser.name()
                ),
            )));
        }
    }

    Ok(outputs)
}
