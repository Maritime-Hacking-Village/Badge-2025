use super::scsi_command_in::ScsiCommandIn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreventAllowMediumRemoval {
    pub prevent: bool,
    /// byte5: Control
    pub control: u8,
}

impl ScsiCommandIn for PreventAllowMediumRemoval {
    const OPCODE: u8 = 0x1E;
    const SIZE: usize = 6;

    fn from_buffer(buffer: &[u8]) -> Self {
        assert!(buffer.len() >= Self::SIZE);
        Self {
            prevent: buffer[19] != 0,
            control: buffer[20],
        }
    }
}
