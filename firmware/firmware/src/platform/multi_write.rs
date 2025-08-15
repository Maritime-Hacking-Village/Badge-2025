use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pipe::Writer};
use embedded_io_async::{ErrorType, Write};

pub struct MultiWrite<W: Write, const MTU: usize> {
    writer1: W,
    writer2: Writer<'static, CriticalSectionRawMutex, MTU>,
}

impl<W, const MTU: usize> ErrorType for MultiWrite<W, MTU>
where
    W: Write,
{
    type Error = W::Error;
}

impl<W: Write, const MTU: usize> MultiWrite<W, MTU> {
    pub fn new(writer1: W, writer2: Writer<'static, CriticalSectionRawMutex, MTU>) -> Self {
        Self { writer1, writer2 }
    }
}

impl<W: Write, const MTU: usize> Write for MultiWrite<W, MTU> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // Write to the first writer
        self.writer1.write_all(buf).await?;

        // Write to the second writer
        // self.writer2.write_all(buf).await.unwrap();
        if let Err(_err) = self.writer2.try_write(buf) {
            // warn!("Error writing to console output buffer: {:?}", err);
        }

        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.writer1.flush().await?;
        self.writer2.flush().await.unwrap();

        Ok(())
    }
}
