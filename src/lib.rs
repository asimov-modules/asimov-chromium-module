// This is free and unencumbered software released into the public domain.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "std")]
extern crate std;

pub mod bookmarks;
pub mod brave;
pub mod jq;
pub mod chrome;
pub mod edge;

pub use bookmarks::*;
