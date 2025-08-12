use alloc::sync::Arc;
use embassy_embedded_hal::shared_bus::{asynch::i2c::I2cDevice, I2cDeviceError};
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_hal::i2c::ErrorType;
use embedded_hal_async::i2c::I2c;

use crate::platform::bq25895::registers::{Register, StatusRegisters};

pub struct Control<M: RawMutex + 'static, DEV: I2c + ErrorType + 'static> {
    pub dev: I2cDevice<'static, M, DEV>, // todo remove pub
    pub status: Arc<Mutex<M, StatusRegisters>>,
}

impl<M: RawMutex, DEV: I2c + ErrorType> Control<M, DEV> {
    pub fn new(dev: I2cDevice<'static, M, DEV>, status: Arc<Mutex<M, StatusRegisters>>) -> Self {
        Control { dev, status }
    }

    pub async fn status(&self) -> StatusRegisters {
        let guard = self.status.lock().await;
        *guard
    }

    pub async fn get_register<REG: Register>(
        &mut self,
    ) -> Result<REG, I2cDeviceError<<DEV as ErrorType>::Error>> {
        let addr = [REG::ADDRESS];
        let mut data = [0];
        self.dev.write_read(0x6a, &addr, &mut data).await?;
        Ok(data[0].into())
    }

    pub async fn set_register<REG: Register>(
        &mut self,
        data: REG,
    ) -> Result<(), I2cDeviceError<<DEV as ErrorType>::Error>> {
        let write_data = [REG::ADDRESS, data.into()];
        self.dev.write(0x6a, &write_data).await?;
        Ok(())
    }

    // TODO: Convenience functions for high power charging and such.
}
