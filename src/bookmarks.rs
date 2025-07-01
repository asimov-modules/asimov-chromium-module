// This is free and unencumbered software released into the public domain.

use jq::{JsonFilter, JsonFilterError};
use serde_json::Value;

/// Transforms Chromium JSON bookmarks to JSON-LD.
pub struct BookmarksTransform {
    filter: JsonFilter,
}

impl BookmarksTransform {
    pub fn new() -> Result<Self, JsonFilterError> {
        Ok(Self {
            filter: crate::jq::BOOKMARKS.parse()?,
        })
    }

    pub fn execute(&self, input: Value) -> Result<Value, JsonFilterError> {
        self.filter.filter_json(input)
    }
}
