use embedded_graphics_transform::{Rotate270, Rotate90};
use embedded_hal::digital::OutputPin;
use mipidsi::{
    interface::{InterfaceAsync, InterfacePixelFormat},
    models::Model,
    DisplayAsync,
};

pub trait VerticalScrolling {
    async fn set_vertical_scroll_region(
        &mut self,
        top_fixed_area: u16,
        bottom_fixed_area: u16,
    ) -> Result<(), ()>;

    async fn set_vertical_scroll_offset(&mut self, offset: u16) -> Result<(), ()>;
}

impl<'a, DI, MODEL, RST> VerticalScrolling for DisplayAsync<'a, DI, MODEL, RST>
where
    DI: InterfaceAsync,
    MODEL: Model,
    MODEL::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    async fn set_vertical_scroll_region(
        &mut self,
        top_fixed_area: u16,
        bottom_fixed_area: u16,
    ) -> Result<(), ()> {
        let _ = self
            .set_vertical_scroll_region(top_fixed_area, bottom_fixed_area)
            .await;
        Ok(())
    }

    async fn set_vertical_scroll_offset(&mut self, offset: u16) -> Result<(), ()> {
        let _ = self.set_vertical_scroll_offset(offset).await;
        Ok(())
    }
}

impl<'a, DI, MODEL, RST> VerticalScrolling for Rotate90<DisplayAsync<'a, DI, MODEL, RST>>
where
    DI: InterfaceAsync,
    MODEL: Model,
    MODEL::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    async fn set_vertical_scroll_region(
        &mut self,
        top_fixed_area: u16,
        bottom_fixed_area: u16,
    ) -> Result<(), ()> {
        let _ = self
            .as_mut()
            .set_vertical_scroll_region(top_fixed_area, bottom_fixed_area)
            .await;
        Ok(())
    }

    async fn set_vertical_scroll_offset(&mut self, offset: u16) -> Result<(), ()> {
        let _ = self.as_mut().set_vertical_scroll_offset(offset).await;
        Ok(())
    }
}

impl<'a, DI, MODEL, RST> VerticalScrolling for Rotate270<DisplayAsync<'a, DI, MODEL, RST>>
where
    DI: InterfaceAsync,
    MODEL: Model,
    MODEL::ColorFormat: InterfacePixelFormat<DI::Word>,
    RST: OutputPin,
{
    async fn set_vertical_scroll_region(
        &mut self,
        top_fixed_area: u16,
        bottom_fixed_area: u16,
    ) -> Result<(), ()> {
        let _ = self
            .as_mut()
            .set_vertical_scroll_region(top_fixed_area, bottom_fixed_area)
            .await;
        Ok(())
    }

    async fn set_vertical_scroll_offset(&mut self, offset: u16) -> Result<(), ()> {
        let _ = self.as_mut().set_vertical_scroll_offset(offset).await;
        Ok(())
    }
}
