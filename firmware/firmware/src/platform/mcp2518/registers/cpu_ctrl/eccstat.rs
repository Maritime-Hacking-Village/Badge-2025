use crate::platform::mcp2518::registers::Register;

#[derive(Clone, Copy, defmt::Format)]
pub struct ECCStat {
    pub erraddr: u16,
    pub dedif: bool,
    pub secif: bool,
}

impl Register for ECCStat {
    const ADDRESS: u16 = 0xE10;
}

impl From<u32> for ECCStat {
    fn from(word: u32) -> Self {
        Self {
            erraddr: ((word >> 16) & 0xFFFF) as u16,
            dedif: word & (1 << 2) != 0,
            secif: word & (1 << 1) != 0,
        }
    }
}

impl From<&ECCStat> for u32 {
    fn from(reg: &ECCStat) -> Self {
        (reg.erraddr as u32) << 16 | (reg.dedif as u32) << 2 | (reg.secif as u32) << 1
    }
}
