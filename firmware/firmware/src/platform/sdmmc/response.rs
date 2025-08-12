use bitfield::Bit;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct R1(pub u8);

impl Default for R1 {
    fn default() -> Self {
        Self(0x80)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum R1Status {
    InIdleState = 0,
    IllegalCommand,
    CommandCRCError,
    EraseSequenceError,
    AddressError,
    ParameterError,
}

impl R1 {
    pub fn valid(self) -> bool {
        !self.0.bit(7)
    }

    pub fn has(self, status: R1Status) -> bool {
        self.0.bit(status as usize)
    }

    pub fn error(self) -> Option<R1Status> {
        let value = (self.0 >> 3) & 0b1111;
        let error_bit = value ^ ((value.wrapping_sub(1)) & value);
        let error = match error_bit {
            0b0001 => R1Status::CommandCRCError,
            0b0010 => R1Status::EraseSequenceError,
            0b0100 => R1Status::AddressError,
            0b1000 => R1Status::ParameterError,
            _ => return None,
        };
        Some(error)
    }
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(C)]
pub struct R3(pub u32);

impl R3 {
    pub fn card_capacity_status(self) -> bool {
        self.0.bit(30)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct R7(pub u32);

impl R7 {
    pub fn voltage_accepted(self) -> bool {
        self.0.bit(8) // only bit 8 meaningful, for now
    }

    pub fn echo_back_check_pattern(self) -> u8 {
        self.0 as u8
    }
}

#[derive(Copy, Clone, Default)]
pub struct Response {
    pub r1: R1,
    pub ex: u32,
}
