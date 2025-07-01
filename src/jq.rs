// This is free and unencumbered software released into the public domain.

pub use ::jq::*;

#[cfg(feature = "std")]
pub fn bookmarks() -> &'static JsonFilter {
    use std::sync::OnceLock;
    static ONCE: OnceLock<JsonFilter> = OnceLock::new();
    ONCE.get_or_init(|| include_str!("jq/bookmarks.jq").parse().unwrap())
}

#[cfg(not(feature = "std"))]
pub fn bookmarks() -> JsonFilter {
    include_str!("jq/bookmarks.jq").parse().unwrap()
}
