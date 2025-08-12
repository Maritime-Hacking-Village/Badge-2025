use crate::platform::bq25895::{registers::Register, value::Value};

pub type IInILim = Value<6, 100, 50, 0x80>;

#[derive(Clone, Copy)]
pub struct Reg00 {
    en_hiz: bool,
    en_ilim: bool,
    i_in_ilim: IInILim,
}

impl Reg00 {
    pub fn new(en_hiz: bool, en_ilim: bool, i_in_ilim: IInILim) -> Self {
        Reg00 {
            en_hiz,
            en_ilim,
            i_in_ilim,
        }
    }
}

impl Default for Reg00 {
    fn default() -> Self {
        Reg00 {
            en_hiz: false,
            en_ilim: true,
            i_in_ilim: IInILim::default(),
        }
    }
}

impl Register for Reg00 {
    const ADDRESS: u8 = 0x00;
}

impl From<u8> for Reg00 {
    fn from(b: u8) -> Self {
        Reg00 {
            en_hiz: (b & 0x80) != 0,
            en_ilim: (b & 0x40) != 0,
            i_in_ilim: IInILim::from(b & 0x3F),
        }
    }
}

impl From<&Reg00> for u8 {
    fn from(reg: &Reg00) -> Self {
        let mut byte = 0;
        byte |= (reg.en_hiz as u8) << 7;
        byte |= (reg.en_ilim as u8) << 6;
        byte |= u8::from(reg.i_in_ilim);
        byte
    }
}

impl From<Reg00> for u8 {
    fn from(reg: Reg00) -> Self {
        u8::from(&reg)
    }
}

impl defmt::Format for Reg00 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg00 {{ 0x{:#08b}: HiZ={}, ilim={}, i_in_ilim={} }}",
            u8::from(self),
            self.en_hiz,
            self.en_ilim,
            self.i_in_ilim,
        )
    }
}
