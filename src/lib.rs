#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

pub use crate::config::get;
pub use crate::build_puzzle::build_puzzle;
pub use crate::verify_puzzle_result::is_puzzle_result_valid;
pub use crate::web::{build_puzzle_service, verify_puzzle_result_service};

pub mod config;
pub mod build_puzzle;
pub mod verify_puzzle_result;

#[cfg(feature = "web")]
pub mod web;
