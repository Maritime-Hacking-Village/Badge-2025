use crate::platform::bq25895::{registers::Register, value::Value};
use defmt::Format;

pub type IChgr = Value<7, 0, 50, 0x00>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg12 {
    pub ichgr: IChgr,
}

impl Reg12 {
    pub fn new(ichgr: IChgr) -> Self {
        Reg12 { ichgr }
    }
}

impl Register for Reg12 {
    const ADDRESS: u8 = 0x12;
}

impl From<u8> for Reg12 {
    fn from(b: u8) -> Self {
        Reg12 {
            ichgr: IChgr::from(b & 0x7F),
        }
    }
}

impl From<&Reg12> for u8 {
    fn from(reg: &Reg12) -> Self {
        let mut byte = 0;
        byte |= u8::from(reg.ichgr);
        byte
    }
}

impl From<Reg12> for u8 {
    fn from(reg: Reg12) -> Self {
        u8::from(&reg)
    }
}

impl core::fmt::Display for Reg12 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Reg12 {{ 0x{:#08b}: IChgr={} }}",
            u8::from(self),
            self.ichgr,
        )
    }
}

impl Format for Reg12 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg12 {{ 0x{:#08b}: IChgr={} }}",
            u8::from(self),
            self.ichgr,
        )
    }
}
