// This is free and unencumbered software released into the public domain.

use std::{io::Result, path::PathBuf};

/// See: https://github.com/chromium/chromium/blob/master/docs/user_data_dir.md
pub fn find_bookmarks_path() -> Result<PathBuf> {
    find_profile_path().map(|path| path.join("Bookmarks"))
}

/// See: https://github.com/chromium/chromium/blob/master/docs/user_data_dir.md
pub fn find_profile_path() -> Result<PathBuf> {
    todo!() // TODO
}
