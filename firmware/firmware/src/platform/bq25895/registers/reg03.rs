use crate::platform::{
    bq25895::{registers::Register, value::Value},
    util::bool_array_to_u8,
};

type SysMin = Value<3, 3000, 100, 0x05>;

#[derive(Clone, Copy)]
pub struct Reg03 {
    pub bat_loaden: bool,
    pub wd_rst: bool,
    pub otg_config: bool,
    pub chg_config: bool,
    pub sys_min: SysMin,
}

impl Reg03 {
    pub fn new(
        bat_loaden: bool,
        wd_rst: bool,
        otg_config: bool,
        chg_config: bool,
        sys_min: SysMin,
    ) -> Self {
        Self {
            bat_loaden,
            wd_rst,
            otg_config,
            chg_config,
            sys_min,
        }
    }
}

impl Default for Reg03 {
    fn default() -> Self {
        Self {
            bat_loaden: false,
            wd_rst: false,
            otg_config: true,
            chg_config: true,
            sys_min: SysMin::default(),
        }
    }
}

impl Register for Reg03 {
    const ADDRESS: u8 = 0x03;
}

impl From<u8> for Reg03 {
    fn from(b: u8) -> Self {
        Reg03 {
            bat_loaden: (b & 0x80) != 0,
            wd_rst: (b & 0x40) != 0,
            otg_config: (b & 0x20) != 0,
            chg_config: (b & 0x10) != 0,
            sys_min: SysMin::from(b >> 1 & 0x07),
        }
    }
}

impl From<&Reg03> for u8 {
    fn from(reg: &Reg03) -> Self {
        bool_array_to_u8(&[
            reg.bat_loaden,
            reg.wd_rst,
            reg.otg_config,
            reg.chg_config,
            false,
            false,
            false,
            false,
        ]) | u8::from(reg.sys_min) << 1
    }
}

impl From<Reg03> for u8 {
    fn from(reg: Reg03) -> Self {
        u8::from(&reg)
    }
}

impl core::fmt::Display for Reg03 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Reg03 {{ {:#08b}: LoadEn: {}, WdRst: {}, OtgConfig: {}, ChgConfig: {}, SysMin: {} }}",
            u8::from(self),
            self.bat_loaden,
            self.wd_rst,
            self.otg_config,
            self.chg_config,
            self.sys_min,
        )
    }
}

impl defmt::Format for Reg03 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg03 {{ {:#08b}: LoadEn: {}, WdRst: {}, OtgConfig: {}, ChgConfig: {}, SysMin: {} }}",
            u8::from(self),
            self.bat_loaden,
            self.wd_rst,
            self.otg_config,
            self.chg_config,
            self.sys_min,
        )
    }
}
