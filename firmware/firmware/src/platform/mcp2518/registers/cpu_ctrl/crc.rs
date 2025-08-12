use crate::platform::mcp2518::registers::Register;

#[derive(Clone, Copy, defmt::Format)]
pub struct CRC {
    pub ferrie: bool,
    pub crcerrie: bool,
    pub ferrif: bool,
    pub crcerrif: bool,
    pub crc: u16,
}

impl Register for CRC {
    const ADDRESS: u16 = 0xE08;
}

impl From<u32> for CRC {
    fn from(word: u32) -> Self {
        Self {
            ferrie: word & (1 << 25) != 0,
            crcerrie: word & (1 << 24) != 0,
            ferrif: word & (1 << 17) != 0,
            crcerrif: word & (1 << 16) != 0,
            crc: word as u16,
        }
    }
}

impl From<&CRC> for u32 {
    fn from(reg: &CRC) -> Self {
        (reg.ferrie as u32) << 25
            | (reg.crcerrie as u32) << 24
            | (reg.ferrif as u32) << 17
            | (reg.crcerrif as u32) << 16
            | reg.crc as u32
    }
}
