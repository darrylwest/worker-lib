#![doc = include_str!("../README.md")]

/// traits and common structs/methods
pub mod worker;

/// concrete implementation
pub mod cache;

/// the current app version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
