use crate::platform::bq25895::{registers::Register, value::Value};

pub type BatV = Value<7, 2304, 20, 0x00>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg0e {
    pub therm_stat: bool,
    pub batv: BatV,
}

impl Reg0e {
    pub fn new(therm_stat: bool, batv: BatV) -> Self {
        Reg0e { therm_stat, batv }
    }
}

impl Register for Reg0e {
    const ADDRESS: u8 = 0x0e;
}

impl From<u8> for Reg0e {
    fn from(b: u8) -> Self {
        Reg0e {
            therm_stat: (b & 0x80) != 0,
            batv: BatV::from(b & 0x7F),
        }
    }
}

impl From<&Reg0e> for u8 {
    fn from(reg: &Reg0e) -> Self {
        let mut byte = 0;
        byte |= (reg.therm_stat as u8) << 7;
        byte |= u8::from(reg.batv);
        byte
    }
}

impl From<Reg0e> for u8 {
    fn from(reg: Reg0e) -> Self {
        u8::from(&reg)
    }
}

impl core::fmt::Display for Reg0e {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Reg0e {{ {:#08b}: ThermStat={}, BatV={} }}",
            u8::from(self),
            self.therm_stat,
            self.batv,
        )
    }
}

impl defmt::Format for Reg0e {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg0e {{ {:#08b}: ThermStat={}, BatV={} }}",
            u8::from(self),
            self.therm_stat,
            self.batv,
        )
    }
}
