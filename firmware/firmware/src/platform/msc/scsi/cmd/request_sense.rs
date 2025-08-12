use byteorder::{ByteOrder, BE};

use super::scsi_command_in::ScsiCommandIn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestSense {
    allocation_length: u8,
}

impl ScsiCommandIn for RequestSense {
    const OPCODE: u8 = 0x03;
    const SIZE: usize = 6;

    fn from_buffer(buffer: &[u8]) -> Self {
        let allocation_length_with_padding = BE::read_u32(&buffer[16..]);
        let allocation_length = allocation_length_with_padding as u8;
        RequestSense { allocation_length }
    }
}
