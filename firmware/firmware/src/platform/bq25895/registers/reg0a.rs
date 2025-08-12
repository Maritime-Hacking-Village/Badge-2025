use crate::platform::bq25895::{registers::Register, value::Value};
use defmt::Format;

type BoostV = Value<4, 4550, 64, 0x09>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg0a {
    boostv: BoostV,
}

impl Reg0a {
    pub fn new(boostv: BoostV) -> Self {
        Self { boostv }
    }
}

impl Register for Reg0a {
    const ADDRESS: u8 = 0x0a;
}

impl From<u8> for Reg0a {
    fn from(b: u8) -> Self {
        Reg0a {
            boostv: BoostV::from(b >> 4 & 0x0F),
        }
    }
}

impl From<&Reg0a> for u8 {
    fn from(reg: &Reg0a) -> Self {
        u8::from(reg.boostv) << 4 | 0x03
    }
}

impl From<Reg0a> for u8 {
    fn from(reg: Reg0a) -> Self {
        u8::from(&reg)
    }
}

impl Format for Reg0a {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg0a {{ 0b{:#08b}: BoostV: {} }}",
            u8::from(self),
            self.boostv,
        )
    }
}
