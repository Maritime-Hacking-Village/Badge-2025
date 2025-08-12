use crate::platform::bq25895::{registers::Register, value::Value};
use defmt::Format;

pub type SysV = Value<7, 2304, 20, 0x00>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg0f {
    pub sysv: SysV,
}

impl Reg0f {
    pub fn new(sysv: SysV) -> Self {
        Reg0f { sysv }
    }
}

impl Register for Reg0f {
    const ADDRESS: u8 = 0x0f;
}

impl From<u8> for Reg0f {
    fn from(b: u8) -> Self {
        Reg0f {
            sysv: SysV::from(b & 0x7F),
        }
    }
}

impl From<&Reg0f> for u8 {
    fn from(reg: &Reg0f) -> Self {
        let mut byte = 0;
        byte |= u8::from(reg.sysv);
        byte
    }
}

impl From<Reg0f> for u8 {
    fn from(reg: Reg0f) -> Self {
        u8::from(&reg)
    }
}

impl Format for Reg0f {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg0f {{ 0x{:#08b}: SysV={} }}",
            u8::from(self),
            self.sysv,
        )
    }
}
