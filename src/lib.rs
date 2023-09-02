#![warn(missing_docs)]
#![warn(unreachable_pub)]

//! # fcaptcha
//!
//! An *experimental* alternative implementation of
//! [FriendlyCaptcha/friendly-lite-server](https://github.com/FriendlyCaptcha/friendly-lite-server)
//! in Rust using [Actix Web](https://actix.rs/).

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

pub use crate::build_puzzle::{build_puzzle, build_puzzle_with};
pub use crate::config::get;
pub use crate::util::get_timestamp;
pub use crate::verify_puzzle_result::{verify_puzzle_result, verify_puzzle_result_with};
#[cfg(feature = "web")]
pub use crate::web::{build_puzzle_service, verify_puzzle_result_service};

/// Implements building puzzles..
pub mod build_puzzle;
/// Implements verifying puzzle results.
pub mod verify_puzzle_result;
/// Implements configuration of the crate.
pub mod config;
/// Implements utility functionality.
pub mod util;

/// Serves the functionality over the web. Requires the `web` feature.
#[cfg(feature = "web")]
pub mod web;
