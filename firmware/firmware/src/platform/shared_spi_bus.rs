//! Asynchronous shared SPI bus with externally controlled CS
//!
//! # Example (nrf52)
//!
//! ```rust,ignore
//! use embassy_embedded_hal::shared_bus::spi::SharedSpiBus;
//! use embassy_sync::mutex::Mutex;
//! use embassy_sync::blocking_mutex::raw::NoopRawMutex;
//!
//! static SPI_BUS: StaticCell<Mutex<NoopRawMutex, spim::Spim<SPI3>>> = StaticCell::new();
//! let mut config = spim::Config::default();
//! config.frequency = spim::Frequency::M32;
//! let spi = spim::Spim::new_txonly(p.SPI3, Irqs, p.P0_15, p.P0_18, config);
//! let spi_bus = Mutex::new(spi);
//! let spi_bus = SPI_BUS.init(spi_bus);
//!
//! // Device 1, using embedded-hal-async compatible driver for ST7735 LCD display
//! let spi_dev1 = SharedSpiBus::new(spi_bus);
//! let display1 = ST7735::new(spi_dev1, dc1, rst1, Default::default(), 160, 128);
//!
//! // Device 2
//! let cs_pin2 = Output::new(p.P0_24, Level::Low, OutputDrive::Standard);
//! let spi_dev2 = SharedSpiBus::new(spi_bus);
//! let display2 = ST7735::new(spi_dev2, dc2, rst2, Default::default(), 160, 128);
//! ```

// use super::locking_spi_bus::LockingSpiBus;
use crate::platform::{locking_spi_bus::LockingSpiBus, set_frequency::SetFrequency};
use embassy_embedded_hal::SetConfig;
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    mutex::{Mutex, MutexGuard},
};
use embedded_hal_async::spi::{ErrorKind, ErrorType, SpiBus};

/// SPI device on a shared bus with externally controlled CS
// pub struct SharedSpiBus<'a, M: RawMutex, SPI>(&'a Mutex<M, SPI>);

// impl<'a, M: RawMutex, BUS> ErrorType for SharedSpiBus<'a, M, BUS>
// where
//     BUS: ErrorType,
// {
//     type Error = BUS::Error;
// }

// impl<'a, M, SPI> LockingSpiBus<'a, M, SPI> for SharedSpiBus<'a, M, SPI>
// where
//     M: RawMutex + 'a,
//     SPI: SpiBus + 'a,
// {
//     async fn lock(&mut self) -> MutexGuard<'a, M, SPI> {
//         self.0.lock().await
//     }
// }

// impl<M, BUS, Word> SpiBus<Word> for SharedSpiBus<'_, M, BUS>
// where
//     M: RawMutex,
//     BUS: SpiBus<Word>,
//     Word: Copy + 'static,
// {
//     async fn read(&mut self, words: &mut [Word]) -> Result<(), Self::Error> {
//         self.0.lock().await.read(words).await
//     }

//     async fn write(&mut self, words: &[Word]) -> Result<(), Self::Error> {
//         self.0.lock().await.write(words).await
//     }

//     async fn transfer(&mut self, read: &mut [Word], write: &[Word]) -> Result<(), Self::Error> {
//         self.0.lock().await.transfer(read, write).await
//     }

//     async fn transfer_in_place(&mut self, words: &mut [Word]) -> Result<(), Self::Error> {
//         self.0.lock().await.transfer_in_place(words).await
//     }

//     async fn flush(&mut self) -> Result<(), Self::Error> {
//         self.0.lock().await.flush().await
//     }
// }

/// SPI device on a shared bus, with its own configuration and externally controlled CS
///
/// This is like [`SharedSpiBus`], with an additional bus configuration that's applied
/// to the bus before each use using [`SetConfig`]. This allows different
/// devices on the same bus to use different communication settings.
pub struct SharedSpiBusWithConfig<'a, M: RawMutex, SPI: SetConfig> {
    bus: &'a Mutex<M, SPI>,
    config: SPI::Config,
}

impl<M, SPI> ErrorType for SharedSpiBusWithConfig<'_, M, SPI>
where
    SPI: ErrorType + SetConfig,
    M: RawMutex,
{
    type Error = SPI::Error;
}

impl<'a, M, SPI> LockingSpiBus<'a, M, SPI> for SharedSpiBusWithConfig<'a, M, SPI>
where
    M: RawMutex + 'a,
    SPI: SpiBus + SetConfig + 'a,
{
    async fn lock(&mut self) -> MutexGuard<'a, M, SPI> {
        let mut ret = self.bus.lock().await;
        ret.set_config(&self.config)
            .map_err(|_| ErrorKind::Other)
            .unwrap();
        ret
    }
}

impl<'a, M, SPI> SharedSpiBusWithConfig<'a, M, SPI>
where
    M: RawMutex,
    SPI: SetConfig,
    SPI::Config: Clone,
{
    /// Create a new `SharedSpiBusWithConfig`.
    pub fn new(bus: &'a Mutex<M, SPI>, config: SPI::Config) -> Self {
        Self { bus, config }
    }

    /// Change the device's config at runtime
    pub fn set_config(&mut self, config: SPI::Config) {
        self.config = config;
    }
}

impl<M, SPI> SetConfig for SharedSpiBusWithConfig<'_, M, SPI>
where
    M: RawMutex,
    SPI: SetConfig,
    SPI::Config: Clone,
{
    type Config = SPI::Config;

    type ConfigError = SPI::ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config.clone());
        Ok(())
    }
}

impl<M, SPI> SetFrequency for SharedSpiBusWithConfig<'_, M, SPI>
where
    M: RawMutex,
    SPI: SetConfig,
    SPI::Config: SetFrequency,
{
    fn set_frequency(&mut self, frequency: u32) {
        self.config.set_frequency(frequency);
    }
}

// impl<M, BUS, Word> SpiBus<Word> for SharedSpiBusWithConfig<'_, M, BUS>
// where
//     M: RawMutex,
//     BUS: SpiBus<Word> + SetConfig,
//     Word: Copy + 'static,
// {
//     async fn read(&mut self, words: &mut [Word]) -> Result<(), Self::Error> {
//         let mut bus = self.bus.lock().await;
//         bus.set_config(&self.config)
//             .map_err(|_| ErrorKind::Other)
//             .unwrap();
//         bus.read(words).await
//     }

//     async fn write(&mut self, words: &[Word]) -> Result<(), Self::Error> {
//         let mut bus = self.bus.lock().await;
//         bus.set_config(&self.config)
//             .map_err(|_| ErrorKind::Other)
//             .unwrap();
//         bus.write(words).await
//     }

//     async fn transfer(&mut self, read: &mut [Word], write: &[Word]) -> Result<(), Self::Error> {
//         let mut bus = self.bus.lock().await;
//         bus.set_config(&self.config)
//             .map_err(|_| ErrorKind::Other)
//             .unwrap();
//         bus.transfer(read, write).await
//     }

//     async fn transfer_in_place(&mut self, words: &mut [Word]) -> Result<(), Self::Error> {
//         let mut bus = self.bus.lock().await;
//         bus.set_config(&self.config)
//             .map_err(|_| ErrorKind::Other)
//             .unwrap();
//         bus.transfer_in_place(words).await
//     }

//     async fn flush(&mut self) -> Result<(), Self::Error> {
//         let mut bus = self.bus.lock().await;
//         bus.flush().await
//     }
// }
