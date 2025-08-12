pub trait ScsiCommandIn {
    const OPCODE: u8;
    const SIZE: usize;
    fn from_buffer(buffer: &[u8]) -> Self;
}
