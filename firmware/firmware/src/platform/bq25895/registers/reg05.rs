use crate::platform::bq25895::{registers::Register, value::Value};

type IPreChg = Value<4, 64, 64, 0x01>;
type ITerm = Value<4, 64, 64, 0x03>;

#[derive(Clone, Copy, Default)]
pub struct Reg05 {
    iprechg: IPreChg,
    iterm: ITerm,
}

impl Reg05 {
    pub fn new(iprechg: IPreChg, iterm: ITerm) -> Self {
        Self { iprechg, iterm }
    }
}

impl Register for Reg05 {
    const ADDRESS: u8 = 0x05;
}

impl From<u8> for Reg05 {
    fn from(b: u8) -> Self {
        Reg05 {
            iprechg: IPreChg::from(b >> 4),
            iterm: ITerm::from(b & 0x0F),
        }
    }
}

impl From<&Reg05> for u8 {
    fn from(reg: &Reg05) -> Self {
        u8::from(reg.iprechg) << 4 | u8::from(reg.iterm)
    }
}

impl From<Reg05> for u8 {
    fn from(reg: Reg05) -> Self {
        u8::from(&reg)
    }
}

impl defmt::Format for Reg05 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg05 {{ 0b{:#08b}: IPreChg: {}, ITerm: {} }}",
            u8::from(self),
            self.iprechg,
            self.iterm,
        )
    }
}
