use miette::{IntoDiagnostic, Result, WrapErr, miette};
use phf::phf_map;
use serde_json::Value;
use std::format;
use std::path::PathBuf;
use std::string::{String, ToString};
use std::vec::Vec;

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
    pub fn name(&self) -> &str {
        self.name
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
        self.profile_path(profile_name)
            .map(|path| path.join("Bookmarks"))
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
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
    }
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
};

pub fn get_browser_from_url(url: &str) -> Option<BrowserConfig> {
    SUPPORTED_BROWSERS
        .entries()
        .find(|(_, config)| url.starts_with(config.url_prefix))
        .map(|(name, paths)| BrowserConfig { name, paths })
}

pub fn fetch_bookmarks(url: &str) -> Result<Vec<Value>> {
    let browser = get_browser_from_url(url).ok_or_else(|| {
        miette!(
            "Unsupported URL: {}. Supported prefixes: {:?}",
            url,
            SUPPORTED_BROWSERS
                .into_iter()
                .map(|(_, config)| config.url_prefix)
                .collect::<Vec<_>>()
        )
    })?;

    let profile_name = url
        .strip_prefix(browser.paths.url_prefix)
        .and_then(|suffix| suffix.strip_prefix('/').filter(|s| !s.is_empty()));

    let mut outputs = Vec::new();

    if let Some(profile) = profile_name {
        let path = browser.bookmarks_path(Some(profile))?;
        outputs.push(read_bookmarks_file(&path)?);
    } else {
        let profiles = browser.list_profiles()?;
        for profile in profiles {
            if let Ok(path) = browser.bookmarks_path(Some(&profile)) {
                if let Ok(bookmarks) = read_bookmarks_file(&path) {
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
    }

    Ok(outputs)
}

fn read_bookmarks_file(path: &PathBuf) -> Result<Value> {
    if !path.is_file() {
        return Err(miette!("Bookmarks file not found at {}", path.display()));
    }

    let input = std::fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read bookmarks at {}", path.display()))?;

    serde_json::from_str(&input)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to parse bookmarks JSON from {}", path.display()))
}
