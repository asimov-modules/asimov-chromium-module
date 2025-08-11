// This is free and unencumbered software released into the public domain.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "std")]
extern crate std;

pub mod bookmarks;
pub mod browsers;
pub mod jq;
pub mod specialized;

pub use bookmarks::*;
