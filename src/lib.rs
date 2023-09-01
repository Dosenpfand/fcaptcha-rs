#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

pub use crate::build_puzzle::{build_puzzle, build_puzzle_with};
pub use crate::config::get;
pub use crate::util::get_timestamp;
pub use crate::verify_puzzle_result::{verify_puzzle_result, verify_puzzle_result_with};
pub use crate::web::{build_puzzle_service, verify_puzzle_result_service};

pub mod build_puzzle;
pub mod config;
pub mod util;
pub mod verify_puzzle_result;

#[cfg(feature = "web")]
pub mod web;
