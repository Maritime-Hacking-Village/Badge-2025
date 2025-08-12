use crate::platform::{
    async_io_on_sync_io::AsyncOutputPin, msc::class::MassStorageClass, sdmmc::spi::SdCard,
};

use super::{irqs::Irqs, msc::class as msc, shared_spi_bus::SharedSpiBusWithConfig};
use embassy_rp::{
    gpio::Output,
    peripherals::{SPI0, USB},
    spi::{Async, Spi},
    usb::Driver,
    Peri,
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_usb::{
    class::cdc_acm::{self, CdcAcmClass},
    UsbDevice,
};
use static_cell::StaticCell;

pub fn initialize(
    peri: Peri<'static, USB>,
) -> (
    UsbDevice<'static, Driver<'static, USB>>,
    CdcAcmClass<'static, Driver<'static, USB>>,
    CdcAcmClass<'static, Driver<'static, USB>>,
    MassStorageClass<
        'static,
        Driver<'static, USB>,
        SdCard<
            'static,
            NoopRawMutex,
            Spi<'static, SPI0, Async>,
            SharedSpiBusWithConfig<'static, NoopRawMutex, Spi<'static, SPI0, Async>>,
            AsyncOutputPin<Output<'static>>,
        >,
    >,
) {
    let driver = Driver::new(peri, Irqs);
    // Create embassy-usb Config
    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("USB-serial example");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config
    };

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let builder = embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 64]),
        );
        builder
    };

    // Create classes on the builder.
    let logger = {
        static STATE: StaticCell<cdc_acm::State> = StaticCell::new();
        let state = STATE.init(cdc_acm::State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    let cli = {
        static STATE: StaticCell<cdc_acm::State> = StaticCell::new();
        let state = STATE.init(cdc_acm::State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    let storage = {
        static STATE: StaticCell<msc::State> = StaticCell::new();
        let state = STATE.init(msc::State::new());

        MassStorageClass::new(&mut builder, state)
    };

    // Build the USB device.
    let usb = builder.build();

    (usb, logger, cli, storage)
}

#[embassy_executor::task]
pub async fn task(mut usb: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    usb.run().await
}
