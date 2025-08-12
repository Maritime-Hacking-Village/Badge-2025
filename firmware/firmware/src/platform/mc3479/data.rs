use defmt::Format;

#[derive(Default, Clone, Copy, Debug, Format, PartialEq, Eq)]
pub struct Data {
    // pub device_status: u8,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub status: StatusRegister,
    pub interrupt_status: InterruptStatusRegister,
}

#[derive(Default, Clone, Copy, Debug, Format, PartialEq, Eq)]
pub struct StatusRegister {
    pub tilt: bool,
    pub flip: bool,
    pub anym: bool,
    pub shake: bool,
    pub tilt35: bool,
    pub fifo: bool,
    pub new_data: bool,
}

impl From<u8> for StatusRegister {
    fn from(value: u8) -> Self {
        StatusRegister {
            tilt: value & 0b00000001 != 0,
            flip: value & 0b00000010 != 0,
            anym: value & 0b00000100 != 0,
            shake: value & 0b00001000 != 0,
            tilt35: value & 0b00010000 != 0,
            fifo: value & 0b00100000 != 0,
            new_data: value & 0b10000000 != 0,
        }
    }
}

#[derive(Default, Clone, Copy, Debug, Format, PartialEq, Eq)]
pub struct InterruptStatusRegister {
    pub tilt_int: bool,
    pub flip_int: bool,
    pub anym_int: bool,
    pub shake_int: bool,
    pub tilt35_int: bool,
    pub fifo_int: bool,
    pub acq_int: bool,
}

impl From<u8> for InterruptStatusRegister {
    fn from(value: u8) -> Self {
        InterruptStatusRegister {
            tilt_int: value & 0b00000001 != 0,
            flip_int: value & 0b00000010 != 0,
            anym_int: value & 0b00000100 != 0,
            shake_int: value & 0b00001000 != 0,
            tilt35_int: value & 0b00010000 != 0,
            fifo_int: value & 0b00100000 != 0,
            acq_int: value & 0b10000000 != 0,
        }
    }
}
