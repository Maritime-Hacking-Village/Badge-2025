use crate::platform::bq25895::{registers::Register, value::Value};

pub type TsPct = Value<7, 21000, 465, 0x00>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg10 {
    pub tspct: TsPct,
}

impl Reg10 {
    pub fn new(tspct: TsPct) -> Self {
        Reg10 { tspct }
    }
}

impl Register for Reg10 {
    const ADDRESS: u8 = 0x10;
}

impl From<u8> for Reg10 {
    fn from(b: u8) -> Self {
        Reg10 {
            tspct: TsPct::from(b & 0x7F),
        }
    }
}

impl From<&Reg10> for u8 {
    fn from(reg: &Reg10) -> Self {
        let mut byte = 0;
        byte |= u8::from(reg.tspct);
        byte
    }
}

impl From<Reg10> for u8 {
    fn from(reg: Reg10) -> Self {
        u8::from(&reg)
    }
}

impl defmt::Format for Reg10 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg10 {{ 0x{:#08b}: TsPct={} }}",
            u8::from(self),
            self.tspct,
        )
    }
}
