mod inquiry;
mod mode_sense6;
mod prevent_allow_medium_removal;
mod read10;
mod read_capacity;
mod request_sense;
mod scsi_command_in;
mod scsi_response_out;
mod test_unit_ready;
mod write10;

use defmt::error;
pub use inquiry::{Inquiry, InquiryCommandData};
pub use mode_sense6::ModeSense6;
pub use prevent_allow_medium_removal::PreventAllowMediumRemoval;
pub use read10::Read10;
pub use read_capacity::{ReadCapacity, ReadCapacityResponse};
pub use request_sense::RequestSense;
pub use scsi_command_in::ScsiCommandIn;
pub use scsi_response_out::ScsiResponseOut;
pub use test_unit_ready::TestUnitReady;
pub use write10::Write10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScsiCommand {
    TestUnitReady(TestUnitReady),
    RequestSense(RequestSense),
    Inquiry(Inquiry),
    ModeSense6(ModeSense6),
    StartStopUnit,
    PreventAllowMediumRemoval(PreventAllowMediumRemoval),
    ReadFormatCapacities,
    ReadCapacity(ReadCapacity),
    Read10(Read10),
    Write10(Write10),
    Verify10,
}

impl ScsiCommand {
    pub fn from_buffer(buffer: &[u8]) -> Result<Self, ()> {
        let opcode = buffer[15];
        Ok(match opcode {
            TestUnitReady::OPCODE => {
                // debug!("TestUnitReady");
                ScsiCommand::TestUnitReady(TestUnitReady::from_buffer(buffer))
            }
            Inquiry::OPCODE => {
                // debug!("Inquiry");
                ScsiCommand::Inquiry(Inquiry::from_buffer(buffer))
            }
            RequestSense::OPCODE => {
                // debug!("RequestSense");
                ScsiCommand::RequestSense(RequestSense::from_buffer(buffer))
            }
            Read10::OPCODE => {
                // debug!("Read10");
                ScsiCommand::Read10(Read10::from_buffer(buffer))
            }
            ReadCapacity::OPCODE => {
                // debug!("ReadCapacity");
                ScsiCommand::ReadCapacity(ReadCapacity::from_buffer(buffer))
            }
            ModeSense6::OPCODE => {
                // debug!("ModeSense6");
                ScsiCommand::ModeSense6(ModeSense6::from_buffer(buffer))
            }
            Write10::OPCODE => {
                // debug!("Write10");
                ScsiCommand::Write10(Write10::from_buffer(buffer))
            }
            PreventAllowMediumRemoval::OPCODE => {
                // debug!("PreventAllowMediumRemoval");
                ScsiCommand::PreventAllowMediumRemoval(PreventAllowMediumRemoval::from_buffer(
                    buffer,
                ))
            }
            op => {
                error!("INVALID OPCODE {:x}", op);
                return Err(());
            }
        })
    }
}
