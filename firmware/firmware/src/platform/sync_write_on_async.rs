//! A utility for synchronous writing operations on asynchronous I/O interfaces.
//!
//! This module provides a bridge between the synchronous `core::fmt::Write` trait and
//! asynchronous I/O operations, allowing code that requires a synchronous writer to
//! work with asynchronous I/O interfaces. It uses an internal channel for buffering.

use alloc::string::{String, ToString};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

/// Adapter that enables synchronous writing to asynchronous I/O interfaces.
///
/// This struct provides a synchronous `Write` implementation that internally buffers
/// data in a channel and asynchronously writes it to the underlying I/O device.
///
/// # Type Parameters
///
/// * `'d`: Lifetime of the channel reference
/// * `T`: An asynchronous writer that implements `embedded_io_async::Write`
/// * `N`: Size of the internal channel buffer
pub struct SyncWriteOnAsync<'d, T, const N: usize>
where
    T: embedded_io_async::Write,
{
    /// The asynchronous I/O device to write to
    pub io: T,
    /// Channel for buffering data between sync and async contexts
    pub channel: &'d Channel<CriticalSectionRawMutex, String, N>,
}

impl<'d, T, const N: usize> SyncWriteOnAsync<'d, T, N>
where
    T: embedded_io_async::Write,
{
    /// Continuously processes and writes data from the channel to the underlying I/O device.
    ///
    /// This method runs in an infinite loop, waiting for data to arrive in the channel
    /// and then writing it to the I/O device. It should be called in an async context.
    ///
    /// # Returns
    ///
    /// This method never returns (marked with `!` return type).
    ///
    /// # Panics
    ///
    /// Panics if writing to the I/O device fails.
    pub async fn run(&mut self) -> ! {
        loop {
            let data = self.channel.receive().await;
            self.io.write_all(data.as_bytes()).await.unwrap();
        }
    }
}

impl<'d, T, const N: usize> SyncWriteOnAsync<'d, T, N>
where
    T: embedded_io_async::Write,
{
    /// Creates a new `SyncWriteOnAsync` instance.
    ///
    /// # Parameters
    ///
    /// * `io`: The asynchronous I/O device to write to
    /// * `channel`: A channel for buffering data between sync and async contexts
    ///
    /// # Returns
    ///
    /// A new `SyncWriteOnAsync` instance.
    pub fn new(io: T, channel: &'d Channel<CriticalSectionRawMutex, String, N>) -> Self {
        Self { io, channel }
    }
}

impl<'d, T, const N: usize> core::fmt::Write for SyncWriteOnAsync<'d, T, N>
where
    T: embedded_io_async::Write,
{
    /// Implements the synchronous `write_str` method required by `core::fmt::Write`.
    ///
    /// This method attempts to enqueue the provided string to the internal channel.
    /// If the channel is full, it returns an error.
    ///
    /// # Parameters
    ///
    /// * `str`: The string to be written
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the string was successfully enqueued
    /// * `Err(core::fmt::Error)` if the channel is full
    fn write_str(&mut self, str: &str) -> Result<(), core::fmt::Error> {
        self.channel
            .try_send(str.to_string())
            .map_err(|_| core::fmt::Error)
    }
}
