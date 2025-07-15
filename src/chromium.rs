// This is free and unencumbered software released into the public domain.

use std::{io::Result, path::PathBuf};

/// See: https://chromium.googlesource.com/chromium/src/+/main/docs/user_data_dir.md
pub fn find_bookmarks_path(profile_name: Option<&str>) -> Result<PathBuf> {
    find_profile_path(profile_name).map(|path| path.join("Bookmarks"))
}

/// See: https://chromium.googlesource.com/chromium/src/+/main/docs/user_data_dir.md
#[cfg(unix)]
pub fn find_profile_path(profile_name: Option<&str>) -> Result<PathBuf> {
    let mut path: PathBuf = getenv::home().expect("HOME must be set").into();

    #[cfg(target_os = "linux")]
    path.push(".config/chromium");

    #[cfg(target_os = "macos")]
    path.push("Library/Application Support/Chromium");

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    todo!(); // TODO

    path.push(profile_name.unwrap_or("Default"));
    Ok(path)
}

#[cfg(not(unix))]
pub fn find_profile_path(profile_name: Option<&str>) -> Result<PathBuf> {
    let mut path: PathBuf = std::env::var("LOCALAPPDATA")
        .expect("LOCALAPPDATA must be set")
        .into();
    path.push("Chromium/User Data");
    path.push(profile_name.unwrap_or("Default"));
    Ok(path)
}
