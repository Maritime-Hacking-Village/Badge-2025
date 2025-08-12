use crate::platform::i2c_io_expander::model::ExpanderModel;
use embedded_hal_async::i2c::I2c;
use itertools::Itertools;

pub struct PCA9536 {}

impl PCA9536 {
    pub const PIN_PWR_INJECTOR: usize = 0;
    pub const PIN_PWR_TRX: usize = 0;
    pub const PIN_PWR_RX: usize = 0;
    pub const PIN_SAO_2: usize = 0;

    pub const N_PINS: usize = 4;
    const ADDRESS: u8 = 0x41;
    const REG_LEN: usize = 1;
    const REG_INIT: [u8; Self::REG_LEN] = [0u8; Self::REG_LEN];
    const INPUT_ADDR: u8 = 0x00;
    const OUTPUT_ADDR: u8 = 0x01;
    const DIRECTION_ADDR: u8 = 0x03;

    // Define methods here
    async fn set_bit<D: I2c>(dev: &mut D, addr: u8, bit: u8, value: bool) {
        let reg = [addr];
        let mut data = [0x00];

        // read
        dev.write_read(Self::ADDRESS, &reg, &mut data)
            .await
            .unwrap();

        // modify
        data[0] &= !(1 << bit);
        data[0] |= (value as u8) << bit;

        // update
        let buf = [reg, data].concat();
        dev.write(Self::ADDRESS, &buf).await.unwrap();
    }
}

impl ExpanderModel for PCA9536 {
    type Inputs = [bool; Self::N_PINS];

    // Sets the direction of the pin (true = input, false = output)
    async fn set_direction<D: I2c>(dev: &mut D, id: u8, direction: bool) {
        let offset = id / 8;
        let bit = id % 8;

        // 1 = input, 0 = output
        Self::set_bit(dev, Self::DIRECTION_ADDR + offset, bit, direction).await;
    }

    async fn set_output<D: I2c>(dev: &mut D, id: u8, output: bool) {
        let offset = id / 8;
        let bit = id % 8;
        Self::set_bit(dev, Self::OUTPUT_ADDR + offset, bit, output).await;
    }

    async fn read_inputs<D: I2c>(dev: &mut D) -> Self::Inputs {
        let reg = [Self::INPUT_ADDR];
        let mut data = Self::REG_INIT;

        // read
        dev.write_read(Self::ADDRESS, &reg, &mut data)
            .await
            .unwrap();

        let values: Self::Inputs = [false; Self::N_PINS];

        values
            .iter()
            .enumerate()
            .map(|(i, _)| data[i / 8] & (1 << (i % 8)) != 0)
            .collect_array()
            .unwrap()
    }
}
