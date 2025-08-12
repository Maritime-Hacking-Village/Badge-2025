use crate::platform::bq25895::{registers::Register, value::Value};

pub type VinDpmOs = Value<5, 0, 100, 0x05>;

#[derive(Clone, Copy, Default)]
pub struct Reg01 {
    bhot: BHot,
    bcold: bool,
    vindpm_os: VinDpmOs,
}

impl Reg01 {
    pub fn new(bhot: BHot, bcold: bool, vindpm_os: VinDpmOs) -> Self {
        Self {
            bhot,
            bcold,
            vindpm_os,
        }
    }
}

impl Register for Reg01 {
    const ADDRESS: u8 = 0x01;
}

impl From<u8> for Reg01 {
    fn from(byte: u8) -> Self {
        Self {
            bhot: BHot::from_byte(byte >> 6),
            bcold: (byte & 0x20) != 0,
            vindpm_os: VinDpmOs::from(byte & 0x1F),
        }
    }
}

impl From<&Reg01> for u8 {
    fn from(reg: &Reg01) -> Self {
        let mut byte = 0;
        byte |= (reg.bhot as u8) << 6;
        byte |= (reg.bcold as u8) << 5;
        byte |= u8::from(reg.vindpm_os);
        byte
    }
}

impl From<Reg01> for u8 {
    fn from(reg: Reg01) -> Self {
        u8::from(&reg)
    }
}

impl defmt::Format for Reg01 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg01 {{ 0b{:#08b}: BHot={}, BCold={}, vindpm_os={} }}",
            u8::from(self),
            self.bhot,
            match self.bcold {
                true => "80%",
                false => "77%",
            },
            self.vindpm_os,
        )
    }
}

#[derive(Clone, Copy)]
pub enum BHot {
    BHot1 = 0x00,
    BHot0 = 0x01,
    BHot2 = 0x02,
    Disabled = 0x03,
}

impl BHot {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => BHot::BHot1,
            0x01 => BHot::BHot0,
            0x02 => BHot::BHot2,
            _ => BHot::Disabled,
        }
    }
}

impl Default for BHot {
    fn default() -> Self {
        BHot::BHot1
    }
}

impl From<BHot> for u8 {
    fn from(value: BHot) -> Self {
        value.into()
    }
}

impl From<u8> for BHot {
    fn from(value: u8) -> Self {
        value.into()
    }
}

impl defmt::Format for BHot {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "BHot {{ 0b{:b}: {} }}",
            self.clone() as u8,
            match self {
                BHot::BHot1 => "34.75%",
                BHot::BHot0 => "37.75%",
                BHot::BHot2 => "31.25%",
                BHot::Disabled => "Disabled",
            },
        )
    }
}
