use super::scsi_command_in::ScsiCommandIn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestUnitReady;

impl ScsiCommandIn for TestUnitReady {
    const OPCODE: u8 = 0x00;
    const SIZE: usize = 6;

    fn from_buffer(_: &[u8]) -> Self {
        TestUnitReady {}
    }
}
