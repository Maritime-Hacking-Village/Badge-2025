use crate::platform::mcp2518::registers::Register;

#[derive(Clone, Copy, defmt::Format)]
pub struct DevId {
    pub id: u8,
    pub rev: u8,
}

impl Register for DevId {
    const ADDRESS: u16 = 0xE14;
}

impl From<u32> for DevId {
    fn from(word: u32) -> Self {
        Self {
            id: ((word >> 4) & 0xF) as u8,
            rev: (word & 0xF) as u8,
        }
    }
}

impl From<&DevId> for u32 {
    fn from(reg: &DevId) -> Self {
        (reg.id as u32) << 4 | (reg.rev as u32)
    }
}
