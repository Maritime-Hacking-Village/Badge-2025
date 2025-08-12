use crate::platform::bq25895::{registers::Register, value::Value};
use defmt::Format;

pub type IDpmLim = Value<6, 100, 50, 0x00>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg14 {
    reg_rst: bool,
    ico_optimized: bool,
    pn: u8,
    ts_profile: bool,
    dev_rev: u8,
}

impl Reg14 {
    pub fn new(reg_rst: bool, ico_optimized: bool, pn: u8, ts_profile: bool, dev_rev: u8) -> Self {
        Reg14 {
            reg_rst,
            ico_optimized,
            pn,
            ts_profile,
            dev_rev,
        }
    }
}

impl Register for Reg14 {
    const ADDRESS: u8 = 0x14;
}

impl From<u8> for Reg14 {
    fn from(b: u8) -> Self {
        Reg14 {
            reg_rst: (b & 0x80) != 0,
            ico_optimized: (b & 0x40) != 0,
            pn: (b >> 3) & 0x07,
            ts_profile: (b & 0x04) != 0,
            dev_rev: b & 0x03,
        }
    }
}

impl From<&Reg14> for u8 {
    fn from(reg: &Reg14) -> Self {
        let mut byte = 0;
        byte |= (reg.reg_rst as u8) << 7;
        byte |= (reg.ico_optimized as u8) << 6;
        byte |= reg.pn << 3;
        byte |= (reg.ts_profile as u8) << 2;
        byte |= reg.dev_rev;
        byte
    }
}

impl From<Reg14> for u8 {
    fn from(reg: Reg14) -> Self {
        u8::from(&reg)
    }
}

impl Format for Reg14 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg14 {{ 0x{:#08b}: RegRst={}, ICOOpt={}, PN={}, TSProfile={}, DevRev={} }}",
            u8::from(self),
            self.reg_rst,
            self.ico_optimized,
            self.pn,
            self.ts_profile,
            self.dev_rev,
        )
    }
}
