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
pub use crate::verify_puzzle_result::{verify_puzzle_result, verify_puzzle_result_with};
pub use crate::config::get;
pub use crate::util::get_timestamp;
#[cfg(feature = "web")]
pub use crate::web::{build_puzzle_service, verify_puzzle_result_service};

pub mod build_puzzle;
pub mod verify_puzzle_result;
pub mod config;
pub mod util;

#[cfg(feature = "web")]
pub mod web;
