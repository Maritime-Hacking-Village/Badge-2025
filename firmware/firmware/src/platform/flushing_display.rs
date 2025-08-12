use embedded_graphics_transform::{Rotate270, Rotate90};
use embedded_hal::digital::OutputPin;
use mipidsi::{
    interface::{InterfaceAsync, InterfacePixelFormat},
    models::Model,
    DisplayAsync,
};

pub trait FlushingDisplay {
    type Error;
    async fn flush(&mut self) -> Result<(), Self::Error>;
}

impl<DI, M, RST> FlushingDisplay for DisplayAsync<'_, DI, M, RST>
where
    DI: InterfaceAsync,
    M: Model,
    M::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    type Error = DI::Error;
    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush().await
    }
}

impl<'a, DI, M, RST> FlushingDisplay for Rotate90<DisplayAsync<'a, DI, M, RST>>
where
    DI: InterfaceAsync,
    M: Model,
    M::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    type Error = DI::Error;

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.as_mut().flush().await
    }
}

impl<'a, DI, M, RST> FlushingDisplay for Rotate270<DisplayAsync<'a, DI, M, RST>>
where
    DI: InterfaceAsync,
    M: Model,
    M::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    type Error = DI::Error;

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.as_mut().flush().await
    }
}

// impl<DI, M, RST> FlushingDisplay for ColorConverted<'_, DisplayAsync<'_, DI, M, RST>, Rgb888>
// where
//     DI: InterfaceAsync,
//     M: Model,
//     M::ColorFormat: InterfacePixelFormat<DI::Word>,
//     RST: OutputPin,
// {
//     type Error = DI::Error;
//     async fn flush(&mut self) -> Result<(), Self::Error> {
//         self.flush().await
//     }
// }
