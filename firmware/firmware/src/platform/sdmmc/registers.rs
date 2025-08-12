use bitfield::bitfield;

bitfield! {
    #[derive(Copy, Clone, defmt::Format)]
    pub struct CSDv1(u128);
    pub version, _: 127, 126;
    pub max_read_data_block_length, _: 83, 80;
    pub device_size, _: 73, 62;
    pub device_size_multiplier, _: 49, 47;
}

#[derive(Copy, Clone, Debug, defmt::Format)]
pub struct NumBlocks {
    device_size: u32,
    multiplier: u16,
}

impl NumBlocks {
    pub fn device_size(&self) -> u32 {
        self.device_size
    }

    pub fn multiplier(&self) -> u16 {
        self.multiplier
    }
}

impl Into<u64> for NumBlocks {
    fn into(self) -> u64 {
        self.device_size as u64 * self.multiplier as u64
    }
}

impl CSDv1 {
    pub fn num_blocks(&self) -> NumBlocks {
        let multiplier = 1 << (self.device_size_multiplier() + 1);
        NumBlocks {
            device_size: self.device_size() as u32 + 1,
            multiplier,
        }
    }

    pub fn read_block_size_shift(&self) -> u8 {
        self.max_read_data_block_length() as u8
    }
}

bitfield! {
    #[derive(Copy, Clone, defmt::Format)]
    pub struct CSDv2(u128);
    pub device_size, _: 69, 48;
}

impl CSDv2 {
    pub fn num_blocks(&self) -> NumBlocks {
        NumBlocks {
            device_size: (self.device_size() as u32 + 1),
            multiplier: 1024,
        }
    }
}

bitfield! {
    #[derive(Copy, Clone, defmt::Format)]
    pub struct CSDv3(u128);
    pub device_size, _: 75, 48;
}

impl CSDv3 {
    pub fn num_blocks(&self) -> NumBlocks {
        NumBlocks {
            device_size: (self.device_size() as u32 + 1),
            multiplier: 1024,
        }
    }
}

#[derive(Copy, Clone, defmt::Format)]
pub enum CSD {
    V1(CSDv1),
    V2(CSDv2),
    V3(CSDv3),
}

impl CSD {
    pub fn try_from(value: u128) -> Option<CSD> {
        let csd = match CSDv1(value).version() {
            0 => Self::V1(CSDv1(value)),
            1 => Self::V2(CSDv2(value)),
            2 => Self::V3(CSDv3(value)),
            _ => return None,
        };
        Some(csd)
    }

    pub fn num_blocks(&self) -> NumBlocks {
        match self {
            Self::V1(csd) => csd.num_blocks(),
            Self::V2(csd) => csd.num_blocks(),
            Self::V3(csd) => csd.num_blocks(),
        }
    }

    pub fn block_size_shift(&self) -> u8 {
        match self {
            Self::V1(csd) => csd.read_block_size_shift(),
            _ => 9, // 512 bytes
        }
    }
}
