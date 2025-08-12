use crate::platform::bq25895::{registers::Register, value::Value};

type BatCmp = Value<3, 0, 20, 0x00>;
type VClamp = Value<3, 0, 32, 0x00>;

#[derive(Clone, Copy, Default)]
pub struct Reg08 {
    bat_comp: BatCmp,
    vclamp: VClamp,
    treg: TReg,
}

impl Reg08 {
    pub fn new(bat_comp: BatCmp, vclamp: VClamp, treg: TReg) -> Self {
        Self {
            bat_comp,
            vclamp,
            treg,
        }
    }
}

impl Register for Reg08 {
    const ADDRESS: u8 = 0x07;
}

impl From<u8> for Reg08 {
    fn from(byte: u8) -> Self {
        Self {
            bat_comp: BatCmp::from(byte >> 4 & 0x07),
            vclamp: VClamp::from(byte >> 2 & 0x07),
            treg: TReg::from(byte & 0x03),
        }
    }
}

impl From<&Reg08> for u8 {
    fn from(reg: &Reg08) -> Self {
        let mut byte = 0;
        byte |= u8::from(reg.bat_comp) << 4;
        byte |= u8::from(reg.vclamp) << 1;
        byte |= u8::from(reg.treg);
        byte
    }
}

impl From<Reg08> for u8 {
    fn from(reg: Reg08) -> Self {
        u8::from(&reg)
    }
}

impl defmt::Format for Reg08 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg08 {{ 0b{:#08b}: BatComp: {}, VClamp: {}, TReg: {} }}",
            u8::from(self),
            self.bat_comp,
            self.vclamp,
            self.treg
        )
    }
}

#[derive(Clone, Copy)]
pub enum TReg {
    Setting60deg = 0x00,
    Setting80deg = 0x01,
    Setting100deg = 0x02,
    Setting120deg = 0x03,
}

impl TReg {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => TReg::Setting60deg,
            0x01 => TReg::Setting80deg,
            0x02 => TReg::Setting100deg,
            _ => TReg::Setting120deg,
        }
    }
}

impl Default for TReg {
    fn default() -> Self {
        TReg::Setting120deg
    }
}

impl From<TReg> for u8 {
    fn from(value: TReg) -> Self {
        value.into()
    }
}

impl From<u8> for TReg {
    fn from(value: u8) -> Self {
        value.into()
    }
}

impl defmt::Format for TReg {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "TReg {{ 0b{:b}: {} }}",
            self.clone() as u8,
            match self {
                TReg::Setting60deg => "60deg",
                TReg::Setting80deg => "80deg",
                TReg::Setting100deg => "100deg",
                TReg::Setting120deg => "120deg",
            }
        )
    }
}
