use crate::platform::mcp2518::registers::Register;

#[derive(Clone, Copy, defmt::Format)]
pub struct OSC {
    pub sclkrdy: SClkRdy,
    pub oscrdy: OscRdy,
    pub pllrdy: PllRdy,
    pub clkodiv: ClkODiv,
    pub sclkdiv: SClkDiv,
    pub lpmen: LPMEn,
    pub oscdis: OscDis,
    pub pllen: PllEn,
}

impl Register for OSC {
    const ADDRESS: u16 = 0xE00;
}

impl From<u32> for OSC {
    fn from(word: u32) -> Self {
        Self {
            sclkrdy: word.into(),
            oscrdy: word.into(),
            pllrdy: word.into(),
            clkodiv: word.into(),
            sclkdiv: word.into(),
            lpmen: word.into(),
            oscdis: word.into(),
            pllen: word.into(),
        }
    }
}

impl From<&OSC> for u32 {
    fn from(reg: &OSC) -> Self {
        reg.sclkrdy as u32
            | reg.oscrdy as u32
            | reg.pllrdy as u32
            | reg.clkodiv as u32
            | reg.sclkdiv as u32
            | reg.lpmen as u32
            | reg.oscdis as u32
            | reg.pllen as u32
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum SClkRdy {
    SClkDiv1,
    SClkDiv0,
}

impl From<u32> for SClkRdy {
    fn from(word: u32) -> Self {
        match ((word >> 12) & 0x01) != 0 {
            true => SClkRdy::SClkDiv1,
            false => SClkRdy::SClkDiv0,
        }
    }
}

impl From<&SClkRdy> for u32 {
    fn from(reg: &SClkRdy) -> Self {
        match reg {
            SClkRdy::SClkDiv1 => 1 << 12,
            SClkRdy::SClkDiv0 => 0,
        }
    }
}

impl From<SClkRdy> for u32 {
    fn from(reg: SClkRdy) -> Self {
        u32::from(&reg)
    }
}

//////
#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum OscRdy {
    NotReadyOrOff,
    RunningAndStable,
}

impl From<u32> for OscRdy {
    fn from(word: u32) -> Self {
        match ((word >> 10) & 0x01) != 0 {
            true => OscRdy::RunningAndStable,
            false => OscRdy::NotReadyOrOff,
        }
    }
}

impl From<&OscRdy> for u32 {
    fn from(reg: &OscRdy) -> Self {
        match reg {
            OscRdy::RunningAndStable => 1 << 10,
            OscRdy::NotReadyOrOff => 0,
        }
    }
}

impl From<OscRdy> for u32 {
    fn from(reg: OscRdy) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum PllRdy {
    NotReady,
    Locked,
}

impl From<u32> for PllRdy {
    fn from(word: u32) -> Self {
        match ((word >> 8) & 0x01) != 0 {
            true => PllRdy::Locked,
            false => PllRdy::NotReady,
        }
    }
}

impl From<&PllRdy> for u32 {
    fn from(reg: &PllRdy) -> Self {
        match reg {
            PllRdy::Locked => 1 << 8,
            PllRdy::NotReady => 0,
        }
    }
}

impl From<PllRdy> for u32 {
    fn from(reg: PllRdy) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
pub enum ClkODiv {
    DivBy10,
    DivBy4,
    DivBy2,
    DivBy1,
}

impl From<u32> for ClkODiv {
    fn from(b: u32) -> Self {
        match (b >> 5) & 0x03 {
            0b11 => ClkODiv::DivBy10,
            0b10 => ClkODiv::DivBy4,
            0b01 => ClkODiv::DivBy2,
            0b00 => ClkODiv::DivBy1,
            _ => panic!("Invalid value"),
        }
    }
}

impl From<&ClkODiv> for u32 {
    fn from(reg: &ClkODiv) -> Self {
        match reg {
            ClkODiv::DivBy10 => 0b11 << 5,
            ClkODiv::DivBy4 => 0b10 << 5,
            ClkODiv::DivBy2 => 0b01 << 5,
            ClkODiv::DivBy1 => 0b00 << 5,
        }
    }
}

impl From<ClkODiv> for u32 {
    fn from(reg: ClkODiv) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum SClkDiv {
    DivBy1,
    DivBy2,
}

impl From<u32> for SClkDiv {
    fn from(word: u32) -> Self {
        match ((word >> 4) & 0x01) != 0 {
            true => SClkDiv::DivBy2,
            false => SClkDiv::DivBy1,
        }
    }
}

impl From<&SClkDiv> for u32 {
    fn from(reg: &SClkDiv) -> Self {
        match reg {
            SClkDiv::DivBy2 => 1 << 4,
            SClkDiv::DivBy1 => 0,
        }
    }
}

impl From<SClkDiv> for u32 {
    fn from(reg: SClkDiv) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum LPMEn {
    Sleep = 0,
    LowPower = 1,
}

impl From<u32> for LPMEn {
    fn from(word: u32) -> Self {
        match ((word >> 3) & 0x01) != 0 {
            true => LPMEn::LowPower,
            false => LPMEn::Sleep,
        }
    }
}

impl From<&LPMEn> for u32 {
    fn from(reg: &LPMEn) -> Self {
        match reg {
            LPMEn::LowPower => 1 << 3,
            LPMEn::Sleep => 0,
        }
    }
}

impl From<LPMEn> for u32 {
    fn from(reg: LPMEn) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum OscDis {
    Enabled = 0,
    Disabled = 1,
}

impl From<u32> for OscDis {
    fn from(word: u32) -> Self {
        match ((word >> 2) & 0x01) != 0 {
            true => OscDis::Disabled,
            false => OscDis::Enabled,
        }
    }
}

impl From<&OscDis> for u32 {
    fn from(reg: &OscDis) -> Self {
        match reg {
            OscDis::Disabled => 1 << 2,
            OscDis::Enabled => 0,
        }
    }
}

impl From<OscDis> for u32 {
    fn from(reg: OscDis) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum PllEn {
    SysClkFromXtal,
    SysClkFrom10xPll,
}

impl From<u32> for PllEn {
    fn from(word: u32) -> Self {
        match (word & 0x01) != 0 {
            true => PllEn::SysClkFrom10xPll,
            false => PllEn::SysClkFromXtal,
        }
    }
}

impl From<&PllEn> for u32 {
    fn from(reg: &PllEn) -> Self {
        match reg {
            PllEn::SysClkFrom10xPll => 1,
            PllEn::SysClkFromXtal => 0,
        }
    }
}

impl From<PllEn> for u32 {
    fn from(reg: PllEn) -> Self {
        u32::from(&reg)
    }
}
