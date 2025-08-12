use crate::platform::mc3479::{
    control::Control,
    data::Data,
    runner::{Runner, ShakeSender},
};
use alloc::sync::Arc;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_hal_async::i2c::I2c;

pub mod control;
pub mod data;
pub mod registers;
pub mod runner;

pub fn setup<MUTEX, DEV>(
    bus: &'static Mutex<MUTEX, DEV>,
    shake_tx: ShakeSender,
) -> (Control<MUTEX, DEV>, Runner<MUTEX, DEV>)
where
    MUTEX: RawMutex,
    DEV: I2c,
{
    let data = Arc::new(Mutex::new(Data::default()));
    (
        Control::new(I2cDevice::new(bus), data.clone()),
        Runner::new(I2cDevice::new(bus), data, shake_tx),
    )
}
