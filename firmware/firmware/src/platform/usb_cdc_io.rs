use core::cmp::min;
use defmt::Format;
use embassy_rp::{peripherals::USB, usb::Driver};
use embassy_usb::{class::cdc_acm::CdcAcmClass, driver::EndpointError};
use embedded_io_async::{Error, ErrorKind, ErrorType, Read, Write};

pub struct UsbCdcIo<'d>(pub CdcAcmClass<'d, Driver<'d, USB>>);

impl<'d> Write for UsbCdcIo<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let len = min(buf.len(), self.0.max_packet_size() as usize);
        // if DTR  is false, we just drop the data
        if self.0.dtr() {
            self.0.write_packet(&buf[..len]).await?;
        }
        Ok(len)
    }
}

impl<'d> Read for UsbCdcIo<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(self.0.read_packet(buf).await?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Format)]
pub struct UsbCdcIoError(EndpointError);

impl<'d> ErrorType for UsbCdcIo<'d> {
    type Error = UsbCdcIoError;
}

impl Error for UsbCdcIoError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl From<EndpointError> for UsbCdcIoError {
    fn from(err: EndpointError) -> Self {
        UsbCdcIoError(err)
    }
}
