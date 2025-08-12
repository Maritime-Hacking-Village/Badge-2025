use crate::platform::bq25895::{registers::Register, value::Value};
use defmt::Format;

pub type VinDpm = Value<7, 2600, 100, 0x12>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg0d {
    force_vindpm: bool,
    vindpm: VinDpm,
}

impl Reg0d {
    pub fn new(force_vindpm: bool, vindpm: VinDpm) -> Self {
        Reg0d {
            force_vindpm,
            vindpm,
        }
    }
}

impl Register for Reg0d {
    const ADDRESS: u8 = 0x0d;
}

impl From<u8> for Reg0d {
    fn from(b: u8) -> Self {
        Reg0d {
            force_vindpm: (b & 0x80) != 0,
            vindpm: VinDpm::from(b & 0x7F),
        }
    }
}

impl From<&Reg0d> for u8 {
    fn from(reg: &Reg0d) -> Self {
        let mut byte = 0;
        byte |= (reg.force_vindpm as u8) << 7;
        byte |= u8::from(reg.vindpm);
        byte
    }
}

impl From<Reg0d> for u8 {
    fn from(reg: Reg0d) -> Self {
        u8::from(&reg)
    }
}

impl Format for Reg0d {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg0d {{ 0x{:#08b}: Force={}, VInDpm={} }}",
            u8::from(self),
            self.force_vindpm,
            self.vindpm,
        )
    }
}
