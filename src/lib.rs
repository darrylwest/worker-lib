#![doc = include_str!("../README.md")]

pub mod cache_worker;
pub mod worker;

/// the current app version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
