use crate::platform::i2c_io_expander::model::ExpanderModel;
use embedded_hal_async::i2c::I2c;
use itertools::Itertools;

pub struct TCAL9539 {}

impl TCAL9539 {
    pub const PIN_JOY_UP: usize = 0;
    pub const PIN_JOY_RIGHT: usize = 1;
    pub const PIN_JOY_DOWN: usize = 2;
    pub const PIN_JOY_CENTER: usize = 3;
    pub const PIN_JOY_LEFT: usize = 4;
    pub const PIN_BUTTON_A: usize = 5;
    pub const PIN_BUTTON_B: usize = 6;
    pub const PIN_DISP_RST: usize = 7;
    pub const PIN_SAO_1: usize = 8;
    pub const PIN_TX_DISCONNECT: usize = 9;
    pub const PIN_TX_DISABLE: usize = 10;
    pub const PIN_CAN_DISCONNECT: usize = 11;
    pub const PIN_SD_CD: usize = 12;
    pub const PIN_RX_TERM_SEL0: usize = 13;
    pub const PIN_RX_TERM_SEL1: usize = 14;
    pub const PIN_RX_TX_TIE: usize = 15;

    pub const N_PINS: usize = 16;
    const ADDRESS: u8 = 0x74;
    const REG_LEN: usize = 2;
    const REG_INIT: [u8; Self::REG_LEN] = [0u8; Self::REG_LEN];
    const INPUT_ADDR: u8 = 0x00;
    const OUTPUT_ADDR: u8 = 0x02;
    const DIRECTION_ADDR: u8 = 0x06;
    const INTERRUPT_MASK_ADDR: u8 = 0x4a;

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

impl ExpanderModel for TCAL9539 {
    type Inputs = [bool; Self::N_PINS];

    // Sets the direction of the pin (true = input, false = output)
    async fn set_direction<D: I2c>(dev: &mut D, id: u8, direction: bool) {
        let offset = id / 8;
        let bit = id % 8;

        // 1 = input, 0 = output
        Self::set_bit(dev, Self::DIRECTION_ADDR + offset, bit, direction).await;

        // 1 = masked, 0 = unmasked (input only)
        Self::set_bit(dev, Self::INTERRUPT_MASK_ADDR + offset, bit, !direction).await;
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
