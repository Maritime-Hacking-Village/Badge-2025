use crate::platform::bq25895::{registers::Register, value::Value};

type VReg = Value<6, 3840, 16, 0x13>;

#[derive(Clone, Copy)]
pub struct Reg06 {
    pub vreg: VReg,
    pub batlowv: bool,
    pub vrechg: bool,
}

impl Reg06 {
    pub fn new(vreg: VReg, batlowv: bool, vrechg: bool) -> Self {
        Self {
            vreg,
            batlowv,
            vrechg,
        }
    }
}

impl Default for Reg06 {
    fn default() -> Self {
        Self {
            vreg: VReg::default(),
            batlowv: true,
            vrechg: false,
        }
    }
}

impl Register for Reg06 {
    const ADDRESS: u8 = 0x06;
}

impl From<u8> for Reg06 {
    fn from(b: u8) -> Self {
        Reg06 {
            vreg: VReg::from(b >> 2),
            batlowv: (b & 0x02) != 0,
            vrechg: (b & 0x01) != 0,
        }
    }
}

impl From<&Reg06> for u8 {
    fn from(reg: &Reg06) -> Self {
        u8::from(reg.vreg) << 2 | (reg.batlowv as u8) << 1 | (reg.vrechg as u8) << 0
    }
}

impl From<Reg06> for u8 {
    fn from(reg: Reg06) -> Self {
        u8::from(&reg)
    }
}

impl defmt::Format for Reg06 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg06 {{ 0b{:#08b}: VReg: {}, BattLowV: {}, VRechg: {} }}",
            u8::from(self),
            self.vreg,
            match self.batlowv {
                true => "3.0V",
                false => "2.8V",
            },
            match self.vrechg {
                true => "200mV",
                false => "100mV",
            },
        )
    }
}
