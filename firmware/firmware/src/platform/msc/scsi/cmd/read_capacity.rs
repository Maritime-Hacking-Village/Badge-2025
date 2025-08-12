use byteorder::{ByteOrder, BE};

use super::{scsi_command_in::ScsiCommandIn, ScsiResponseOut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadCapacity;

impl ScsiCommandIn for ReadCapacity {
    const OPCODE: u8 = 0x25;
    const SIZE: usize = 10;

    fn from_buffer(_: &[u8]) -> Self {
        ReadCapacity {}
    }
}

/// Response from an executed ReadCapacityCommand with capacity information.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ReadCapacityResponse {
    /// The address of the final block on the disk.
    ///
    /// The device therefore has a size of (1 + `logical_block_address`) * `block_length`
    /// bytes, since the block addresses start at 0.
    pub logical_block_address: u32,

    /// The number of bytes in a single block for this device.
    pub block_length: u32,
}

impl ScsiResponseOut for ReadCapacityResponse {
    const SIZE: usize = 8;

    fn to_buffer(&self, buffer: &mut [u8]) -> Result<(), ()> {
        let buffer = buffer.as_mut();
        BE::write_u32(&mut buffer[0..], self.logical_block_address);
        BE::write_u32(&mut buffer[4..], self.block_length);
        Ok(())
    }
}
