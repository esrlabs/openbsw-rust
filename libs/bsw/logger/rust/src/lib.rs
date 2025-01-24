#![no_std]
//! This crate provides a logger, which forwards all log messages where the target matches a
//! bsw logger component.
//! Other log messages with an unknown target are directly printed to stdout.
//!
//! To initialize and register the logger call `::logger::add_component_info_array` and ``::logger::install_rust_logging` from cpp.
//!
//! # Map entire crates to a logger component
//!
//! It's possible to map entire crates to one logger component by calling ``::logger::map_crate_to_component``.
//! This allows the omission of a target for all log messages from the specified crate.
//!
//! # Examples
//! ```
//! // Send log to CONSOLE Logger Component
//! info!(target: "CONSOLE", "This message is coming from {}", "Rust");
//!
//! // Write to stdout if logger component is unknown
//! warn!(target: "unknown", "This will be printed on stdout directly")
//!
//! // This only ends up in a logger component if a crate to component mapping was configured
//! info!(
//!   "Without a given target the log crate will use a crate/module name combination as the target"
//! )
//! ```

mod bsw_logger;
mod ffi;
mod log_buf;

pub use bsw_logger::BswLogger;
