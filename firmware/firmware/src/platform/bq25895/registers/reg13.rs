use crate::platform::bq25895::{registers::Register, value::Value};

pub type IDpmLim = Value<6, 100, 50, 0x00>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg13 {
    pub vdpm_stat: bool,
    pub idpm_stat: bool,
    pub idpm_lim: IDpmLim,
}

impl Reg13 {
    pub fn new(vdpm_stat: bool, idpm_stat: bool, idpm_lim: IDpmLim) -> Self {
        Reg13 {
            vdpm_stat,
            idpm_stat,
            idpm_lim,
        }
    }
}

impl Register for Reg13 {
    const ADDRESS: u8 = 0x13;
}

impl From<u8> for Reg13 {
    fn from(b: u8) -> Self {
        Reg13 {
            vdpm_stat: (b & 0x80) != 0,
            idpm_stat: (b & 0x40) != 0,
            idpm_lim: IDpmLim::from(b & 0x3F),
        }
    }
}

impl From<&Reg13> for u8 {
    fn from(reg: &Reg13) -> Self {
        let mut byte = 0;
        byte |= (reg.vdpm_stat as u8) << 7;
        byte |= (reg.idpm_stat as u8) << 6;
        byte |= u8::from(reg.idpm_lim);
        byte
    }
}

impl From<Reg13> for u8 {
    fn from(reg: Reg13) -> Self {
        u8::from(&reg)
    }
}

impl defmt::Format for Reg13 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg13 {{ 0x{:#08b}: VDpmStat={}, IDpmStat={}, IDpmLim={} }}",
            u8::from(self),
            self.vdpm_stat,
            self.idpm_stat,
            self.idpm_lim,
        )
    }
}
