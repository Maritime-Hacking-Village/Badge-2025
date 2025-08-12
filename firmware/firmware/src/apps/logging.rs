use crate::platform::sync_write_on_async::SyncWriteOnAsync;
use alloc::{format, string::String};
use defmt::back_channel;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, lazy_lock::LazyLock,
};
use embassy_time::Instant;
use embedded_io_async::Write;

const QUEUE_SIZE: usize = 4;

pub struct Logger<'s, W>(SyncWriteOnAsync<'s, W, QUEUE_SIZE>)
where
    W: Write;

impl<'s, W: Write> Logger<'s, W> {
    pub fn new(writer: W) -> Self {
        static CHANNEL: LazyLock<Channel<CriticalSectionRawMutex, String, QUEUE_SIZE>> =
            LazyLock::new(|| Channel::new());

        // configure a callback to receive log messages from back_channel
        // the callback is sync, so it needs to use the channel
        back_channel::set_callback(|s| {
            let now = Instant::now().as_micros();
            let line = format!("{}.{:06} {}\r\n", now / 1000000, now % 1000000, s);

            // Cannot use io, so just get channel from static
            if let Err(_) = CHANNEL.get().try_send(line) {
                // debug!("Failed to send log message to channel");
            }
        });

        Self(SyncWriteOnAsync::new(writer, CHANNEL.get()))
    }

    pub async fn run(&mut self) -> ! {
        self.0.run().await
    }
}
