//! # Back Channel Module
//!
//! This module provides functionality for setting up a custom callback function
//! to receive log messages independently of defmt's standard output.
//!
//! The back channel mechanism allows applications to capture logs through a registered
//! callback function, enabling scenarios such as:
//! - Sending logs over alternative communication channels (UART, BLE, etc.)
//! - Storing logs in memory for later retrieval
//! - Processing log messages in custom ways
//!
//! Note: This module cannot have defmt as a dependency to avoid circular dependencies.

use alloc::string::String;

/// Global static variable holding the optional callback function.
///
/// This is marked as `unsafe` because it's a mutable static variable,
/// which requires unsafe code to access and modify.
static mut CALLBACK: Option<fn(String)> = None;

/// Sets a callback function to receive formatted log messages.
///
/// This function will only set the callback if none has been set previously.
/// Once a callback is set, it cannot be changed or unset.
///
/// # Parameters
///
/// * `callback` - A function that takes a `String` parameter containing the log message
///
/// # Safety
///
/// This function uses unsafe code to access a mutable static variable.
/// It is the caller's responsibility to ensure this function is not called concurrently
/// with other functions that access the `CALLBACK` variable.
pub fn set_callback(callback: fn(String)) {
    unsafe {
        if let None = CALLBACK {
            CALLBACK = Some(callback);
        }
    }
}

/// Retrieves the currently registered callback function, if any.
///
/// # Returns
///
/// * `Option<fn(String)>` - The registered callback function, or `None` if no callback has been set
///
/// # Safety
///
/// This function uses unsafe code to access a mutable static variable.
/// It is the caller's responsibility to ensure this function is not called concurrently
/// with other functions that modify the `CALLBACK` variable.
pub fn get_callback() -> Option<fn(String)> {
    unsafe { CALLBACK }
}
