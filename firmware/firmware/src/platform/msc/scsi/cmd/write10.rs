use byteorder::{ByteOrder, BE};

use super::scsi_command_in::ScsiCommandIn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Write10 {
    /// The start address of the write, in units of device blocks.
    pub block_address: u32,

    /// The number of blocks to write.
    pub transfer_blocks: u16,
}

impl ScsiCommandIn for Write10 {
    const OPCODE: u8 = 0x2A;
    const SIZE: usize = 10;

    fn from_buffer(buffer: &[u8]) -> Self {
        let block_address = BE::read_u32(&buffer[17..]);
        let transfer_blocks = BE::read_u16(&buffer[22..]);
        Write10 {
            block_address,
            transfer_blocks,
        }
    }
}
