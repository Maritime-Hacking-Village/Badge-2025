use crate::platform::bq25895::{registers::Register, value::Value};
use defmt::Format;

pub type VBusV = Value<7, 2600, 100, 0x00>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg11 {
    pub vbus_gd: bool,
    pub vbusv: VBusV,
}

impl Reg11 {
    pub fn new(vbus_gd: bool, vbusv: VBusV) -> Self {
        Reg11 { vbus_gd, vbusv }
    }
}

impl Register for Reg11 {
    const ADDRESS: u8 = 0x11;
}

impl From<u8> for Reg11 {
    fn from(b: u8) -> Self {
        Reg11 {
            vbus_gd: (b & 0x80) != 0,
            vbusv: VBusV::from(b & 0x7F),
        }
    }
}

impl From<&Reg11> for u8 {
    fn from(reg: &Reg11) -> Self {
        let mut byte = 0;
        byte |= (reg.vbus_gd as u8) << 7;
        byte |= u8::from(reg.vbusv);
        byte
    }
}

impl From<Reg11> for u8 {
    fn from(reg: Reg11) -> Self {
        u8::from(&reg)
    }
}

impl Format for Reg11 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg11 {{ 0x{:#08b}: VbusGd={}, VBusV={} }}",
            u8::from(self),
            self.vbus_gd,
            self.vbusv,
        )
    }
}
