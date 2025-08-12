use byteorder::{ByteOrder, LE};

/// This struct prefaces all responses from the SCSI device when a command
/// requires a response.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CommandStatus {
    /// The value of the `status` field if the initating command succeeded.
    Passed,
    /// The value of the `status` field if the initating command failed.
    Failed,
    /// The value of the `status` field if the initating command encountered a
    /// phase error.
    PhaseError,
}

impl Default for CommandStatus {
    fn default() -> Self {
        CommandStatus::Passed
    }
}

impl Into<u8> for CommandStatus {
    fn into(self) -> u8 {
        match self {
            CommandStatus::Passed => 0,
            CommandStatus::Failed => 1,
            CommandStatus::PhaseError => 2,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub struct CommandStatusWrapper {
    /// The value of the `tag` field of the `CommandBlockWrapper` this CSW
    /// corresponds to.
    pub tag: u32,

    /// Difference between CBW length and actual length.
    pub data_residue: u32,

    /// The status value returned after the command was ran; any value other
    /// than 0 indicates an error.
    ///
    /// See `COMMAND_PASSED`, `COMMAND_FAILED`, and `PHASE_ERROR` for some
    /// known values this field can take.
    pub status: CommandStatus,
}

impl CommandStatusWrapper {
    /// The size of the Command Status Wrapper, including magic number, in bytes.
    pub const SIZE: usize = 13;

    /// A magic number that should preface the Command Status Wrapper on the buffer.
    pub const D_CSW_SIGNATURE: u32 = 0x5342_5355;

    pub fn to_buffer(&self, buffer: &mut [u8; Self::SIZE]) -> Result<(), ()> {
        LE::write_u32(buffer, CommandStatusWrapper::D_CSW_SIGNATURE);
        LE::write_u32(&mut buffer[4..], self.tag);
        LE::write_u32(&mut buffer[8..], self.data_residue);
        buffer[12] = self.status.into();
        Ok(())
    }
}
