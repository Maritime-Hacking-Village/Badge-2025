use byteorder::{ByteOrder, BE};

use super::scsi_command_in::ScsiCommandIn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeSense6 {
    allocation_length: u8,
}

impl ScsiCommandIn for ModeSense6 {
    const OPCODE: u8 = 0x1A;
    const SIZE: usize = 6;

    fn from_buffer(buffer: &[u8]) -> Self {
        let allocation_length_with_padding = BE::read_u32(&buffer[16..]);
        let allocation_length = allocation_length_with_padding as u8;
        ModeSense6 { allocation_length }
    }
}
