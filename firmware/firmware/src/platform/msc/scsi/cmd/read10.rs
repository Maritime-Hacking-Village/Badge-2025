use byteorder::{ByteOrder, BE};

use super::scsi_command_in::ScsiCommandIn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Read10 {
    /// The start address of the read, in units of device blocks.
    pub block_address: u32,

    /// The number of bytes in a single block.
    // pub block_size: u32,

    /// The size of the read, in units of blocks.
    pub transfer_blocks: u16,
}

impl ScsiCommandIn for Read10 {
    const OPCODE: u8 = 0x28;
    const SIZE: usize = 10;

    fn from_buffer(buffer: &[u8]) -> Self {
        let block_address = BE::read_u32(&buffer[17..]);
        let transfer_blocks = BE::read_u16(&buffer[22..]);
        Read10 {
            block_address,
            transfer_blocks,
        }
    }
}
