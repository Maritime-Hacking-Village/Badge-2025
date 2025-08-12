pub trait ScsiResponseOut {
    // const OPCODE: u8;
    const SIZE: usize;
    fn to_buffer(&self, buffer: &mut [u8]) -> Result<(), ()>;
}
