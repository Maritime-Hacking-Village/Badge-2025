//! Top-level module for all embassy-executor tasks.

pub mod batt;
pub mod ctrl;
pub mod irq;
pub mod log;
pub mod repl;
pub mod rx;
pub mod tx;
#[cfg(feature = "wifi")]
pub mod wifi;
