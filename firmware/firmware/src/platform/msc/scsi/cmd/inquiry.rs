use byteorder::{ByteOrder, BE};

use super::{ScsiCommandIn, ScsiResponseOut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inquiry {
    allocation_length: u8,
}

impl ScsiCommandIn for Inquiry {
    const OPCODE: u8 = 0x12;
    const SIZE: usize = 6;

    fn from_buffer(buffer: &[u8]) -> Self {
        let allocation_length_with_padding = BE::read_u32(&buffer[16..]);
        let allocation_length = allocation_length_with_padding as u8;
        Inquiry { allocation_length }
    }
}

/// The data sent in response to an `InquiryCommand`.
///
/// Currently does not include all data that the device responds with; finishing
/// this is a TODO item.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub struct InquiryResponse {
    /// 3 bit flag set to determine the SCSI device's current accessibility. For
    /// most common devices, this will be 0 to indicate that the device is accessible
    /// for running commands.
    ///
    /// The flag bits are as follows:
    ///
    /// * If the least significant bit, `0x20`, is set, then the "SCSI task router"
    /// in use is currently unable to access the logical unit specified via the LUN
    /// field in the CBW.
    ///
    /// * If the next bit, `0x40`, is set, then specified LUN cannot be accessed
    /// by the current "SCSI task router". Note that this flag being set implies
    /// that the previous bit must also be set, and that the returned value of
    /// `device_type` is `0x1F` to indicate a value of `Unknown`.
    ///
    /// * If the most significant bit, `0x80`, is set, then this flat set must be
    /// interpretted through the device vendor's documentation instead of the information
    /// given here; the previous 2 bullets no longer apply.
    pub device_qualifier: u8,

    /// The type of SCSI device this is. Usually 0 to indicate a
    /// randomly accessable block storage device.
    ///
    /// For valid values, see the [SCSI Peripheral Device Type](https://en.wikipedia.org/wiki/SCSI_Peripheral_Device_Type)
    /// list.
    pub device_type: u8,

    /// A flag set whose most significant bit corresponds to whether or not the
    /// device is removable.
    ///
    /// Currently the next bit corresponds to whether or not the logical unit
    /// is part of a "logical unit conglomerate", while the rest are reserved for
    /// future use.
    pub removable_flags: u8,

    /// Indicates what version of the SCSI command set this device conforms to;
    /// at time of writing, a value of 7 corresponds to adherence to the latest
    /// version of the command specifications, SPC-5.
    ///
    /// A value of 0 indicates that the device does not claim to match *any*
    /// SCSI specification version; proceed with caution in that case.
    pub spc_version: u8,

    /// What format the response will be in.
    ///
    /// Currently, the only valid value is 2.
    pub response_format: u8,
}

impl ScsiResponseOut for InquiryResponse {
    const SIZE: usize = 31;

    fn to_buffer(&self, buffer: &mut [u8]) -> Result<(), ()> {
        let buffer = buffer.as_mut();
        let bt = self.device_qualifier | self.device_type;
        buffer[0] = bt;
        buffer[1] = self.removable_flags;
        buffer[2] = self.spc_version;
        buffer[3] = self.response_format;
        Ok(())
    }
}

/// SCSI Inquiry command structure
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub struct InquiryCommandData {
    // byte0
    pub peripheral_qualifier: u8,
    pub peripheral_device_type: u8,
    // byte1
    pub rmb: bool,
    // byte2
    pub version: u8,
    // byte3
    pub aerc: bool,
    pub normaca: bool,
    pub hisup: bool,
    pub response_data_format: u8,
    // byte4
    pub additional_length: u8,
    // byte5
    pub sccs: bool,
    // byte6
    pub bque: bool,
    pub encserv: bool,
    pub vs0: bool,
    pub multip: bool,
    pub mchngr: bool,
    pub addr16: bool,
    // byte7
    pub reladr: bool,
    pub wbus16: bool,
    pub sync: bool,
    pub linked: bool,
    pub cmdque: bool,
    pub vs1: bool,
    // byte8-15
    pub vendor_id: [u8; 8],
    // byte16-31
    pub product_id: [u8; 16],
    // byte32-35
    pub product_revision_level: [u8; 4],
}

impl InquiryCommandData {
    pub fn new(vendor_id: [u8; 8], product_id: [u8; 16], product_revision_level: [u8; 4]) -> Self {
        Self {
            peripheral_qualifier: 0,
            peripheral_device_type: 0,
            rmb: true,
            version: 0x4,
            aerc: false,
            normaca: false,
            hisup: false,
            response_data_format: 0x2,
            additional_length: 0x1f,
            sccs: false,
            bque: false,
            encserv: false,
            vs0: false,
            multip: false,
            mchngr: false,
            addr16: false,
            reladr: false,
            wbus16: false,
            sync: false,
            linked: false,
            cmdque: false,
            vs1: false,
            vendor_id,
            product_id,
            product_revision_level,
        }
    }
}
impl ScsiResponseOut for InquiryCommandData {
    const SIZE: usize = 36;

    fn to_buffer(&self, buf: &mut [u8]) -> Result<(), ()> {
        assert!(buf.len() >= Self::SIZE);

        buf[0] = (self.peripheral_qualifier << 5) | (self.peripheral_device_type & 0x1f);
        buf[1] = (self.rmb as u8) << 7;
        buf[2] = self.version;
        buf[3] = ((self.aerc as u8) << 7)
            | ((self.normaca as u8) << 5)
            | ((self.hisup as u8) << 4)
            | (self.response_data_format & 0xf);
        buf[4] = self.additional_length;
        buf[5] = (self.sccs as u8) << 0x1;
        buf[6] = ((self.bque as u8) << 7)
            | ((self.encserv as u8) << 6)
            | ((self.vs0 as u8) << 5)
            | ((self.multip as u8) << 4)
            | ((self.mchngr as u8) << 3)
            | ((self.addr16 as u8) << 1);
        buf[7] = ((self.reladr as u8) << 7)
            | ((self.wbus16 as u8) << 6)
            | ((self.sync as u8) << 5)
            | ((self.linked as u8) << 4)
            | ((self.cmdque as u8) << 1)
            | (self.vs1 as u8);
        buf[8..16].copy_from_slice(&self.vendor_id);
        buf[16..32].copy_from_slice(&self.product_id);
        buf[32..36].copy_from_slice(&self.product_revision_level);
        Ok(())
    }
}
