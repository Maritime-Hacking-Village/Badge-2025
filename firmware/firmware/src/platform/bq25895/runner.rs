use alloc::sync::Arc;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_hal_async::i2c::I2c;

use crate::platform::{
    bq25895::registers::{
        reg0b::Reg0b, reg0c::Reg0c, reg0e::Reg0e, reg0f::Reg0f, reg10::Reg10, reg11::Reg11,
        reg12::Reg12, StatusRegisters,
    },
    interrupt_i2c::update_state::UpdateState,
};

pub struct Runner<M: RawMutex + 'static, D: I2c + 'static> {
    dev: I2cDevice<'static, M, D>,
    status: Arc<Mutex<M, StatusRegisters>>,
}
impl<M: RawMutex + 'static, D: I2c + 'static> Runner<M, D> {
    pub fn new(dev: I2cDevice<'static, M, D>, status: Arc<Mutex<M, StatusRegisters>>) -> Self {
        Self { dev, status }
    }

    // pub async fn run(&self) {
    //     let mut status = self.status.lock().await;
    //     // TODO: Implement runner logic
    // }
    //
    pub async fn read_inputs(&mut self) {
        // read 0x0b
        let reg = [0x0b];
        let mut data_0b = [0u8; 1];
        self.dev.write_read(0x6a, &reg, &mut data_0b).await.unwrap();

        // read 0x0c
        let reg = [0x0c];
        let mut data_0c = [0u8; 1];
        self.dev.write_read(0x6a, &reg, &mut data_0c).await.unwrap();
        self.dev.write_read(0x6a, &reg, &mut data_0c).await.unwrap();

        // Read 5 registers starting at 0x0e
        let reg = [0x0e];
        let mut data = [0u8; 5];
        self.dev.write_read(0x6a, &reg, &mut data).await.unwrap();

        // update status
        let mut status = self.status.lock().await;
        status.reg0b = Reg0b::from(data_0b[0]);
        status.reg0c = Reg0c::from(data_0c[0]);

        status.reg0e = Reg0e::from(data[0]);
        status.reg0f = Reg0f::from(data[1]);
        status.reg10 = Reg10::from(data[2]);
        status.reg11 = Reg11::from(data[3]);
        status.reg12 = Reg12::from(data[4]);
    }
}

impl<M: RawMutex, D: I2c> UpdateState for Runner<M, D> {
    async fn update(&mut self) {
        self.read_inputs().await;
    }
}
