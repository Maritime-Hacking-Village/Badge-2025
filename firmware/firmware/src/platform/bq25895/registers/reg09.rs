use crate::platform::{bq25895::registers::Register, util::bool_array_to_u8};
use defmt::Format;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reg09 {
    force_ico: bool,
    tmr2x_en: bool,
    batfet_dis: bool,
    batfet_dly: bool,
    batfet_rst_en: bool,
    pumpx_up: bool,
    pumpx_down: bool,
}

impl Reg09 {
    pub fn new(
        force_ico: bool,
        tmr2x_en: bool,
        batfet_dis: bool,
        batfet_dly: bool,
        batfet_rst_en: bool,
        pumpx_up: bool,
        pumpx_down: bool,
    ) -> Self {
        Self {
            force_ico,
            tmr2x_en,
            batfet_dis,
            batfet_dly,
            batfet_rst_en,
            pumpx_up,
            pumpx_down,
        }
    }
}

impl Default for Reg09 {
    fn default() -> Self {
        Self {
            force_ico: false,
            tmr2x_en: true,
            batfet_dis: false,
            batfet_dly: false,
            batfet_rst_en: true,
            pumpx_up: false,
            pumpx_down: false,
        }
    }
}

impl Register for Reg09 {
    const ADDRESS: u8 = 0x09;
}

impl From<u8> for Reg09 {
    fn from(b: u8) -> Self {
        Reg09 {
            force_ico: (b & 0x80) != 0,
            tmr2x_en: (b & 0x40) != 0,
            batfet_dis: (b & 0x20) != 0,
            batfet_dly: (b & 0x08) != 0,
            batfet_rst_en: (b & 0x04) != 0,
            pumpx_up: (b & 0x02) != 0,
            pumpx_down: (b & 0x01) != 0,
        }
    }
}

impl From<&Reg09> for u8 {
    fn from(reg: &Reg09) -> Self {
        bool_array_to_u8(&[
            reg.force_ico,
            reg.tmr2x_en,
            reg.batfet_dis,
            false,
            reg.batfet_dly,
            reg.batfet_rst_en,
            reg.pumpx_up,
            reg.pumpx_down,
        ])
    }
}

impl From<Reg09> for u8 {
    fn from(reg: Reg09) -> Self {
        u8::from(&reg)
    }
}

impl Format for Reg09 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg09 {{ 0b{:#08b}: ForceICO {}, TMR2xEn={}, BatFETDis={}, BatFETDly={}, BatFETRstEn={}, PumpXUp={}, PumpXDown={} }}",
            u8::from(self),
            self.force_ico,
            self.tmr2x_en,
            self.batfet_dis,
            self.batfet_dly,
            self.batfet_rst_en,
            self.pumpx_up,
            self.pumpx_down,
        )
    }
}
