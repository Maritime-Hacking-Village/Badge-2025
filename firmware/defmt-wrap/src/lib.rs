#![no_std]
//! # defmt-wrap
//!
//! This crate provides a wrapper around the [`defmt`](https://github.com/knurling-rs/defmt) crate,
//! extending its functionality by allowing log messages to be sent to both the standard defmt
//! output and a custom back channel.
//!
//! ## Overview
//!
//! The `defmt-wrap` crate reexports all the functionality of the original `defmt` crate while
//! enhancing its logging capabilities. With this wrapper, log messages are simultaneously sent to:
//!
//! 1. The standard defmt output (typically RTT or ITM)
//! 2. A user-defined callback function that can process the logs in custom ways
//!
//! This dual-channel approach enables scenarios such as:
//! - Sending logs over alternative communication channels (UART, BLE, etc.)
//! - Storing logs in memory for later retrieval or analysis
//! - Processing log messages for real-time monitoring or alerts
//! - Creating redundant logging paths for critical systems
//!
//! ## Usage
//!
//! ```rust
//! use defmt;
//!
//! // Define a callback function to receive log messages
//! fn my_callback(message: String) {
//!     // Process the log message as needed
//!     // For example, send it over UART, BLE, or store in memory
//! }
//!
//! fn main() {
//!     // Register the callback function
//!     defmt::back_channel::set_callback(my_callback);
//!
//!     // Now use defmt logging macros as usual
//!     defmt::info!("System initialized");
//!     defmt::warn!("Low memory: {} bytes remaining", available_memory);
//!     defmt::error!("Failed to initialize device: {}", error_code);
//! }
//! ```
//!
//! The crate reexports all defmt functionality while adding custom logging macros that
//! simultaneously log to the original defmt output and to a user-defined callback function.

/// This crate is designed for `no_std` environments, commonly used in embedded systems.
/// It maintains full compatibility with the original defmt crate while adding
/// the ability to send log messages to a custom callback function.

/// Module providing functionality for setting up a custom callback function
/// to receive log messages independently of defmt's standard output.
///
/// This module allows you to register a callback function that will receive all
/// log messages produced by the logging macros. The callback receives formatted
/// log messages as Strings, which can then be processed as needed by your application.
pub mod back_channel;

/// Re-export of the original defmt crate as "upstream" for internal use
pub use defmt_orig as upstream;
/// Re-export all items from the original defmt crate
pub use defmt_orig::*;
/// Re-export embedded_alloc for memory allocation in no_std environments
pub use embedded_alloc;

/// Re-export useful allocation-related items for convenience
pub use alloc::{
    format,
    string::{String, ToString},
};

/// Import the alloc crate for heap allocations in no_std environments
extern crate alloc;

/// Internal macro used by the logging macros to send formatted log messages
/// to the registered back channel callback function.
///
/// This is an implementation detail that should not be used directly.
/// Instead, use the public logging macros like `info!`, `warn!`, and `error!`.
///
/// # Parameters
///
/// * `$p` - Prefix literal string (usually indicates log level like "I: ", "W: ", etc.)
/// * `$s` - Main message literal string
/// * `$x` - Optional format arguments
///
/// # Safety
///
/// This macro uses unsafe code to access the global callback function.
/// The unsafe block is necessary because the callback is stored in a static mutable
/// variable, which requires unsafe access according to Rust's memory safety rules.
/// However, the implementation ensures that this access is safe by only allowing
/// the callback to be set once during program initialization.
#[macro_export]
macro_rules! write_data {
    ($p:literal, $s:literal $(, $x:expr)* $(,)?) => {{
        unsafe {
            if let Some(callback) = defmt::back_channel::get_callback() {
                callback(defmt::format!(concat!($p, $s) $(, $x)*));
            }
        }
    }};
}

// /// Log a debug message to both defmt and the back channel.
// ///
// /// This macro simultaneously sends the message to the standard defmt debug output
// /// and to the registered back channel callback with a "D: " prefix.
// ///
// /// # Parameters
// ///
// /// * `$s` - Message literal string
// /// * `$x` - Optional format arguments
// #[macro_export]
// macro_rules! debug {
//     ($s:literal $(, $x:expr)* $(,)?) => {{
//         defmt::upstream::debug!($s $(, $x)*);
//         defmt::write_data!("D: ", $s $(, $x)*);
//     }};
// }

/// Log an info message to both defmt and the back channel.
///
/// This macro simultaneously sends the message to the standard defmt info output
/// and to the registered back channel callback with an "I: " prefix.
///
/// # Parameters
///
/// * `$s` - Message literal string
/// * `$x` - Optional format arguments
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use defmt;
///
/// defmt::info!("System initialized with value: {}", 42);
/// ```
///
/// With multiple arguments:
///
/// ```rust
/// use defmt;
///
/// let temperature = 25.5;
/// let humidity = 60;
/// defmt::info!("Current conditions: {}°C, {}% humidity", temperature, humidity);
/// ```
///
/// The messages will be sent to both the standard defmt output and any registered
/// back channel callback.
#[macro_export]
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        defmt::upstream::info!($s $(, $x)*);
        defmt::write_data!("I: ", $s $(, $x)*);
    }};
}

/// Log a warning message to both defmt and the back channel.
///
/// This macro simultaneously sends the message to the standard defmt warn output
/// and to the registered back channel callback with a "W: " prefix.
///
/// # Parameters
///
/// * `$s` - Message literal string
/// * `$x` - Optional format arguments
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use defmt;
///
/// let available_memory = 1024;
/// defmt::warn!("Low memory: only {} bytes remaining", available_memory);
/// ```
///
/// With multiple warning conditions:
///
/// ```rust
/// use defmt;
///
/// let battery = 15;
/// let signal = 2;
/// defmt::warn!("System resources critical: battery {}%, signal strength {}/5", battery, signal);
/// ```
///
/// The warning messages will be sent to both the standard defmt output and any registered
/// back channel callback.
#[macro_export]
macro_rules! warn {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        defmt::upstream::warn!($s $(, $x)*);
        defmt::write_data!("W: ", $s $(, $x)*);
    }};
}

/// Log an error message to both defmt and the back channel.
///
/// This macro simultaneously sends the message to the standard defmt error output
/// and to the registered back channel callback with an "E: " prefix.
///
/// # Parameters
///
/// * `$s` - Message literal string
/// * `$x` - Optional format arguments
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use defmt;
///
/// let error_code = 42;
/// defmt::error!("Failed to initialize device: {}", error_code);
/// ```
///
/// With error details:
///
/// ```rust
/// use defmt;
///
/// let operation = "sensor calibration";
/// let error_code = 0x8F;
/// let attempts = 3;
/// defmt::error!("Operation {} failed with code 0x{:02X} after {} attempts",
///               operation, error_code, attempts);
/// ```
///
/// The error messages will be sent to both the standard defmt output and any registered
/// back channel callback.
#[macro_export]
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        defmt::upstream::error!($s $(, $x)*);
        defmt::write_data!("E: ", $s $(, $x)*);
    }};
}
