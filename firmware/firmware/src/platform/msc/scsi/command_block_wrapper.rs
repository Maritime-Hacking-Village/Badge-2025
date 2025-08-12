use alloc::string::String;
use byteorder::{ByteOrder, LE};

use super::cmd::*;

/// A struct that prefaces all commands in the SCSI protocol.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CommandBlockWrapper {
    /// The identifier of the command this CBW is wrapping
    pub tag: u32,

    /// How much non-command data needs to be transfered for this command
    /// (eg data being read for a Read10, data being written for a Write10, etc)
    pub data_transfer_length: u32,

    /// General flags about the command to be executed; currently only supports
    /// the most significant bit, which is 1 when the command requires the device to
    /// send data back to the host and 0 otherwise.
    pub flags: u8,

    /// The logical unit number of the device to execute the command on.
    pub lun: u8,

    /// The length of the command parameters to be executed, not counting external data to be transfered.
    pub cb_length: u8,

    /// The direction data will be flowing in, either IN for device -> host, OUT for host -> device,
    /// and NONE if the command has no associated data transfer.
    // pub direction: Direction,
    pub command: ScsiCommand,
}

impl CommandBlockWrapper {
    /// A magic number that should preface the Command Block Wrapper on the buffer.
    pub const D_CBW_SIGNATURE: u32 = 0x4342_5355;

    pub fn from_buffer<B: AsRef<[u8]>>(buffer: B) -> Result<Self, String> {
        let buffer = buffer.as_ref();
        let magic = LE::read_u32(buffer);

        if magic != CommandBlockWrapper::D_CBW_SIGNATURE {
            return Err("invalid signature".into());
        }

        let tag = LE::read_u32(&buffer[4..]);
        let data_transfer_length = LE::read_u32(&buffer[8..]);

        let flags = buffer[12];
        let lun = buffer[13];
        let cb_length = buffer[14];

        // match opcode
        Ok(Self {
            tag,
            data_transfer_length,
            flags,
            lun,
            cb_length,
            command: ScsiCommand::from_buffer(buffer).map_err(|_| "invalid command")?,
        })
    }
}
