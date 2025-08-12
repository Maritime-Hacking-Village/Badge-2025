pub mod control;
pub mod registers;
pub mod runner;
pub mod value;

use alloc::sync::Arc;
use control::Control;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_hal_async::i2c::I2c;

use crate::platform::bq25895::{registers::StatusRegisters, runner::Runner};

pub fn setup<MUTEX, DEV>(
    bus: &'static Mutex<MUTEX, DEV>,
) -> (Control<MUTEX, DEV>, Control<MUTEX, DEV>, Runner<MUTEX, DEV>)
where
    MUTEX: RawMutex,
    DEV: I2c,
{
    let status = Arc::new(Mutex::new(StatusRegisters::default()));
    (
        Control::new(I2cDevice::new(bus), status.clone()),
        Control::new(I2cDevice::new(bus), status.clone()),
        Runner::new(I2cDevice::new(bus), status),
    )
}
