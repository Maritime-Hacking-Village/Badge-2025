use crate::platform::mcp2518::registers::Register;

#[derive(Clone, Copy, defmt::Format)]
pub struct ECCCon {
    pub parity: u8,
    pub dedie: bool,
    pub secie: bool,
    pub eccen: bool,
}

impl Register for ECCCon {
    const ADDRESS: u16 = 0xE0C;
}

impl From<u32> for ECCCon {
    fn from(word: u32) -> Self {
        Self {
            parity: ((word >> 8) & 0xFF) as u8,
            dedie: word & (1 << 2) != 0,
            secie: word & (1 << 1) != 0,
            eccen: word & 1 != 0,
        }
    }
}

impl From<&ECCCon> for u32 {
    fn from(reg: &ECCCon) -> Self {
        (reg.parity as u32) << 8
            | (reg.dedie as u32) << 2
            | (reg.secie as u32) << 1
            | reg.eccen as u32
    }
}
