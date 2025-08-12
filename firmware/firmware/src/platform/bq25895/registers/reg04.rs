use crate::platform::bq25895::{registers::Register, value::Value};

type IChg = Value<7, 0, 64, 0x20>;

#[derive(Clone, Copy, Default)]
pub struct Reg04 {
    en_pumpx: bool,
    ichg: IChg,
}

impl Reg04 {
    pub fn new(en_pumpx: bool, ichg: IChg) -> Self {
        Self { en_pumpx, ichg }
    }
}

impl Register for Reg04 {
    const ADDRESS: u8 = 0x04;
}

impl From<u8> for Reg04 {
    fn from(b: u8) -> Self {
        Reg04 {
            en_pumpx: (b & 0x80) != 0,
            ichg: IChg::from(b & 0x7F),
        }
    }
}

impl From<&Reg04> for u8 {
    fn from(reg: &Reg04) -> Self {
        (reg.en_pumpx as u8) << 7 | u8::from(reg.ichg)
    }
}

impl From<Reg04> for u8 {
    fn from(reg: Reg04) -> Self {
        u8::from(&reg)
    }
}

impl defmt::Format for Reg04 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg04 {{ 0b{:#08b}: Pump: {}, IChg: {} }}",
            u8::from(self),
            self.en_pumpx,
            self.ichg,
        )
    }
}
