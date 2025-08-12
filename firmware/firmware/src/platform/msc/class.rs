use core::{cmp::min, mem::MaybeUninit};

use crate::platform::{
    async_io_on_sync_io::AsyncOutputPin,
    msc::scsi::{
        cmd::{InquiryCommandData, ReadCapacityResponse, ScsiCommand, ScsiResponseOut},
        command_block_wrapper::CommandBlockWrapper,
        command_status_wrapper::{CommandStatus, CommandStatusWrapper},
    },
    sdmmc::spi::SdCard,
    shared_spi_bus::SharedSpiBusWithConfig,
};

use super::storage::MscStorage;
use alloc::vec::Vec;
use defmt::{debug, error, info, trace, warn};
use embassy_rp::{
    self,
    gpio::Output,
    peripherals::{SPI0, USB},
    spi::{Async, Spi},
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_usb::{
    control::{InResponse, OutResponse, Recipient, Request, RequestType},
    driver::{Driver, Endpoint, EndpointIn, EndpointOut},
    types::InterfaceNumber,
    Builder, Handler,
};

const MSC_INTERFACE_CLASS: u8 = 0x08;
const MSC_INTERFACE_SUBCLASS: u8 = 0x06;
const MSC_INTERFACE_PROTOCOL: u8 = 0x50;

const REQ_MASS_STORAGE_RESET: u8 = 0xFF;
const REQ_MASS_STORAGE_GET_MAX_LUN: u8 = 0xFE;

const USB_LOGICAL_BLOCK_SIZE: usize = 64;

pub struct State {
    control: MaybeUninit<Control>,
}

impl<'a> Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
        }
    }
}

pub struct MassStorageClass<'d, D: Driver<'d>, S: MscStorage> {
    _data_if: InterfaceNumber,
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    storage: Option<S>,
    vendor_id: [u8; 8],
    product_id: [u8; 16],
    product_revision_level: [u8; 4],
}

/// USB Mass Storage Class Control Handler
/// This handler is used to handle the control requests for the Mass Storage Class.
/// It supports the Mass Storage Reset and Get Max LUN requests.
pub struct Control {
    // Interface Number
    // comm_if: InterfaceNumber,
    // Bulk Transfer Request Sender (for Mass Storage Reset)
    // bulk_request_sender: DynamicSender<'c, BulkTransferRequest>,
}

impl Handler for Control {
    fn control_out(&mut self, req: Request, buf: &[u8]) -> Option<OutResponse> {
        debug!("Got control_out, request={}, buf={:a}", req, buf);
        None
    }

    /// Respond to DeviceToHost control messages, where the host requests some data from us.
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        debug!("Got control_in, request={}", req);

        // requestType: Class/Interface, host->device
        // request: 0xff (Mass Storage Reset), 0xfe (Get Max LUN)

        if req.request_type != RequestType::Class || req.recipient != Recipient::Interface {
            return None;
        }
        match req.request {
            x if x == REQ_MASS_STORAGE_RESET => {
                // Mass Storage Reset
                debug!("Mass Storage Reset");
                // match self
                //     .bulk_request_sender
                //     .try_send(BulkTransferRequest::Reset)
                // {
                //     Ok(_) => Some(InResponse::Accepted(&buf[..0])),
                //     Err(_) => Some(InResponse::Rejected),
                // }
                Some(InResponse::Rejected)
                // Some(InResponse::Accepted(&buf[..0]))
            }
            x if x == REQ_MASS_STORAGE_GET_MAX_LUN && req.length == 1 => {
                // Get Max LUN
                debug!("Get Max LUN");
                buf[0] = 0; // Only one LUN supported
                Some(InResponse::Accepted(&buf[..1]))
            }
            _ => {
                warn!("Unsupported request: {}", req.request);
                Some(InResponse::Rejected)
            }
        }
    }
}

impl<'d, D: Driver<'d>, S: MscStorage> MassStorageClass<'d, D, S> {
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State) -> Self {
        let (ifnum, read_ep, write_ep) = {
            let mut func = builder.function(
                MSC_INTERFACE_CLASS,
                MSC_INTERFACE_SUBCLASS,
                MSC_INTERFACE_PROTOCOL,
            );

            // Bulk Only Transport for Mass Storage
            let mut interface = func.interface();
            let ifnum = interface.interface_number();
            let mut alt = interface.alt_setting(
                MSC_INTERFACE_CLASS,
                MSC_INTERFACE_SUBCLASS,
                MSC_INTERFACE_PROTOCOL,
                None,
            );
            let read_ep = alt.endpoint_bulk_out(None, 64);
            let write_ep = alt.endpoint_bulk_in(None, 64);
            (ifnum, read_ep, write_ep)
        };

        let control = state.control.write(Control {});
        builder.handler(control);

        Self {
            _data_if: ifnum,
            read_ep,
            write_ep,
            storage: None,
            vendor_id: *b"EmbassyU",
            product_id: *b"USB Mass Storage",
            product_revision_level: *b"1.00",
        }
    }

    pub fn set_storage(&mut self, storage: S) {
        self.storage = Some(storage);
    }
}

impl<'d, D: Driver<'d>, S: MscStorage> MassStorageClass<'d, D, S> {
    // Main loop for bulk-only transport
    pub async fn run(&mut self) -> ! {
        // assert!(self.read_ep.is_some());
        // assert!(self.write_ep.is_some());
        // let read_ep = self.read_ep.as_mut().unwrap();
        // let write_ep = self.write_ep.as_mut().unwrap();
        loop {
            self.read_ep.wait_enabled().await;
            debug!("Connected");

            // // Request Sense Command Error reporting
            // let mut latest_sense_data: Option<RequestSenseData> = None;
            // // Phase Error
            // let mut phase_error_tag: Option<u32> = None;

            'read_ep_loop: loop {
                // Check if Mass Storage Reset occurred
                // if (self.ctrl_to_bulk_request_receiver.try_receive()
                //     == Ok(BulkTransferRequest::Reset))
                // {
                //     debug!("Mass Storage Reset");
                //     phase_error_tag = None;
                //     break;
                // }

                // clear latest sense data
                // latest_sense_data = None;

                // Command Transport
                let mut read_buf = [0u8; USB_LOGICAL_BLOCK_SIZE]; // read buffer
                let Ok(_) = self.read_ep.read(&mut read_buf).await else {
                    error!("Read EP Error (CBW)");
                    // phase_error_tag = None; // unknown tag
                    // latest_sense_data = Some(RequestSenseData::from(
                    //     SenseKey::IllegalRequest,
                    //     AdditionalSenseCodeType::IllegalRequestInvalidCommand,
                    // ));
                    break 'read_ep_loop;
                };
                trace!("Read buffer: {:?}", read_buf);

                let Ok(cbw) = CommandBlockWrapper::from_buffer(&read_buf) else {
                    break 'read_ep_loop;
                };

                match cbw.command {
                    ScsiCommand::Inquiry(_) => {
                        debug!("Inquiry");

                        let inquiry_response = InquiryCommandData::new(
                            self.vendor_id,
                            self.product_id,
                            self.product_revision_level,
                        );
                        let mut response_buffer = [0; InquiryCommandData::SIZE];
                        inquiry_response.to_buffer(&mut response_buffer).unwrap();
                        self.write_ep.write(&response_buffer).await.unwrap();

                        // write CSW
                        let csw = CommandStatusWrapper {
                            tag: cbw.tag,
                            data_residue: 0,
                            status: CommandStatus::Passed,
                        };

                        let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                        csw.to_buffer(&mut csw_buffer).unwrap();
                        self.write_ep.write(&csw_buffer).await.unwrap();
                    }
                    ScsiCommand::TestUnitReady(_) => {
                        if let Some(storage) = &self.storage {
                            if storage.is_present() {
                                debug!("TestUnitReady - Ready");
                                // write CSW - success
                                let csw = CommandStatusWrapper {
                                    tag: cbw.tag,
                                    data_residue: 0,
                                    status: CommandStatus::Passed,
                                };

                                let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                                csw.to_buffer(&mut csw_buffer).unwrap();
                                self.write_ep.write(&csw_buffer).await.unwrap();
                            } else {
                                debug!("TestUnitReady - No media present");
                                // Report that media is not present
                                let csw = CommandStatusWrapper {
                                    tag: cbw.tag,
                                    data_residue: 0,
                                    status: CommandStatus::Failed,
                                };

                                let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                                csw.to_buffer(&mut csw_buffer).unwrap();
                                self.write_ep.write(&csw_buffer).await.unwrap();
                            }
                        } else {
                            // debug!("TestUnitReady - No storage");
                            // Report that the unit is not ready
                            let csw = CommandStatusWrapper {
                                tag: cbw.tag,
                                data_residue: 0,
                                status: CommandStatus::Failed,
                            };

                            let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                            csw.to_buffer(&mut csw_buffer).unwrap();
                            self.write_ep.write(&csw_buffer).await.unwrap();
                        }
                    }
                    ScsiCommand::RequestSense(_) => {
                        if let Some(storage) = &self.storage {
                            if !storage.is_present() {
                                debug!("RequestSense - Media not present");

                                // Return standard "media not present" sense data
                                // Format: Current error, Medium not present, etc.
                                let sense_data = [
                                    0x70, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00,
                                    0x00, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x00, 0x00,
                                ];

                                self.write_ep.write(&sense_data).await.unwrap();
                            } else {
                                debug!("RequestSense - No error");

                                // Return no error
                                let sense_data = [
                                    0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00,
                                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                ];

                                self.write_ep.write(&sense_data).await.unwrap();
                            }
                        } else {
                            // debug!("RequestSense - No storage");

                            // Return standard "media not present" sense data
                            let sense_data = [
                                0x70, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00,
                                0x00, 0x3A, 0x00, 0x00, 0x00, 0x00, 0x00,
                            ];

                            self.write_ep.write(&sense_data).await.unwrap();
                        }

                        // write CSW
                        let csw = CommandStatusWrapper {
                            tag: cbw.tag,
                            data_residue: 0,
                            status: CommandStatus::Passed,
                        };

                        let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                        csw.to_buffer(&mut csw_buffer).unwrap();
                        self.write_ep.write(&csw_buffer).await.unwrap();
                    }
                    ScsiCommand::ReadCapacity(_) => {
                        if let Some(storage) = &self.storage {
                            if storage.is_present() {
                                debug!("ReadCapacity");

                                // Get capacity from storage
                                let read_capacity_data = storage.get_capacity();

                                debug!(
                                    "Capacity: LBA({}) block_size({})",
                                    read_capacity_data.logical_block_address,
                                    read_capacity_data.block_length
                                );

                                let mut write_data = [0u8; ReadCapacityResponse::SIZE];
                                read_capacity_data.to_buffer(&mut write_data).unwrap();
                                self.write_ep.write(&write_data).await.unwrap();

                                // write CSW
                                let csw = CommandStatusWrapper {
                                    tag: cbw.tag,
                                    data_residue: 0,
                                    status: CommandStatus::Passed,
                                };

                                let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                                csw.to_buffer(&mut csw_buffer).unwrap();
                                self.write_ep.write(&csw_buffer).await.unwrap();
                            } else {
                                debug!("ReadCapacity - No media present");

                                // Return media not present error
                                let csw = CommandStatusWrapper {
                                    tag: cbw.tag,
                                    data_residue: cbw.data_transfer_length,
                                    status: CommandStatus::Failed,
                                };

                                let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                                csw.to_buffer(&mut csw_buffer).unwrap();
                                self.write_ep.write(&csw_buffer).await.unwrap();
                            }
                        } else {
                            debug!("ReadCapacity - No storage");

                            // Return error
                            let csw = CommandStatusWrapper {
                                tag: cbw.tag,
                                data_residue: cbw.data_transfer_length,
                                status: CommandStatus::Failed,
                            };

                            let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                            csw.to_buffer(&mut csw_buffer).unwrap();
                            self.write_ep.write(&csw_buffer).await.unwrap();
                        }
                    }

                    ScsiCommand::ModeSense6(_) => {
                        debug!("ModeSense6 Command");

                        let write_data = [0x03u8, 0, 0, 0];
                        self.write_ep.write(&write_data).await.unwrap();

                        // write CSW
                        let csw = CommandStatusWrapper {
                            tag: cbw.tag,
                            data_residue: 0,
                            status: CommandStatus::Passed,
                        };

                        let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                        csw.to_buffer(&mut csw_buffer).unwrap();
                        self.write_ep.write(&csw_buffer).await.unwrap();
                    }
                    ScsiCommand::Read10(cmd) => {
                        info!(
                            "Read10Command addr({:x}), size({}) blocks({})",
                            cmd.block_address, 512, cmd.transfer_blocks
                        );

                        if let Some(storage) = &mut self.storage {
                            if storage.is_present() {
                                // Read data from storage
                                match storage.read_blocks(&cmd).await {
                                    Ok(blocks) => {
                                        // Send blocks to host
                                        for block in blocks {
                                            // Send 512-byte blocks in chunks that fit in the USB packet size
                                            let mut remaining: usize = 512;
                                            let mut offset: usize = 0;

                                            while remaining > 0 {
                                                let to_send: usize = min(remaining, 64);
                                                let slice = &block[offset..offset + to_send];
                                                trace!(
                                                    "SD data - remaining {}: Writing {} bytes",
                                                    remaining,
                                                    to_send
                                                );
                                                self.write_ep.write(slice).await.unwrap();

                                                remaining -= to_send;
                                                offset += to_send;
                                            }
                                        }

                                        // write success CSW
                                        let csw = CommandStatusWrapper {
                                            tag: cbw.tag,
                                            data_residue: 0,
                                            status: CommandStatus::Passed,
                                        };

                                        let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                                        csw.to_buffer(&mut csw_buffer).unwrap();
                                        self.write_ep.write(&csw_buffer).await.unwrap();
                                    }
                                    Err(_) => {
                                        error!("Failed to read from storage");

                                        // write failure CSW
                                        let csw = CommandStatusWrapper {
                                            tag: cbw.tag,
                                            data_residue: cbw.data_transfer_length,
                                            status: CommandStatus::Failed,
                                        };

                                        let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                                        csw.to_buffer(&mut csw_buffer).unwrap();
                                        self.write_ep.write(&csw_buffer).await.unwrap();
                                    }
                                }
                            } else {
                                debug!("Read10 - No media present");

                                // Return media not present error
                                let csw = CommandStatusWrapper {
                                    tag: cbw.tag,
                                    data_residue: cbw.data_transfer_length,
                                    status: CommandStatus::Failed,
                                };

                                let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                                csw.to_buffer(&mut csw_buffer).unwrap();
                                self.write_ep.write(&csw_buffer).await.unwrap();
                            }
                        } else {
                            debug!("Read10 - No storage");

                            // Return error
                            let csw = CommandStatusWrapper {
                                tag: cbw.tag,
                                data_residue: cbw.data_transfer_length,
                                status: CommandStatus::Failed,
                            };

                            let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                            csw.to_buffer(&mut csw_buffer).unwrap();
                            self.write_ep.write(&csw_buffer).await.unwrap();
                        }
                    }
                    ScsiCommand::PreventAllowMediumRemoval(_) => {
                        // write CSW
                        let csw = CommandStatusWrapper {
                            tag: cbw.tag,
                            data_residue: 0,
                            status: CommandStatus::Passed,
                        };

                        let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                        csw.to_buffer(&mut csw_buffer).unwrap();
                        self.write_ep.write(&csw_buffer).await.unwrap();
                    }

                    ScsiCommand::Write10(cmd) => {
                        info!("Write10Command");

                        if let Some(storage) = &mut self.storage {
                            if storage.is_present() {
                                let block_size = 512;
                                let mut blocks = Vec::new();

                                // Read data from host into blocks
                                for _ in 0..cmd.transfer_blocks {
                                    let mut block = [0u8; 512];
                                    let mut remaining = block_size;
                                    let mut offset = 0;

                                    while remaining > 0 {
                                        let to_read = min(remaining, 64);
                                        let mut buffer = [0u8; 64];
                                        let read_size = self
                                            .read_ep
                                            .read(&mut buffer[..to_read])
                                            .await
                                            .unwrap();

                                        block[offset..offset + read_size]
                                            .copy_from_slice(&buffer[..read_size]);

                                        offset += read_size;
                                        remaining -= read_size;
                                    }

                                    blocks.push(block);
                                }

                                // Write blocks to storage
                                match storage.write_blocks(&cmd, &blocks).await {
                                    Ok(_) => {
                                        info!(
                                            "Successfully wrote {} blocks to storage",
                                            cmd.transfer_blocks
                                        );

                                        // write success CSW
                                        let csw = CommandStatusWrapper {
                                            tag: cbw.tag,
                                            data_residue: 0,
                                            status: CommandStatus::Passed,
                                        };

                                        let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                                        csw.to_buffer(&mut csw_buffer).unwrap();
                                        self.write_ep.write(&csw_buffer).await.unwrap();
                                    }
                                    Err(_) => {
                                        error!("Failed to write to storage");

                                        // write failure CSW
                                        let csw = CommandStatusWrapper {
                                            tag: cbw.tag,
                                            data_residue: 0, // We consumed all the data
                                            status: CommandStatus::Failed,
                                        };

                                        let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                                        csw.to_buffer(&mut csw_buffer).unwrap();
                                        self.write_ep.write(&csw_buffer).await.unwrap();
                                    }
                                }
                            } else {
                                debug!("Write10 - No media present");

                                // We need to consume the data the host wants to send
                                // since a full transfer is expected, but then report error
                                let block_size = 512;
                                let total_size = block_size * cmd.transfer_blocks as usize;
                                let mut remaining = total_size;

                                // Read and discard the data
                                while remaining > 0 {
                                    let to_read = min(remaining, 64);
                                    let mut buffer = [0u8; 64];
                                    match self.read_ep.read(&mut buffer[..to_read]).await {
                                        Ok(read_size) => {
                                            remaining -= read_size;
                                        }
                                        Err(e) => {
                                            error!("Error reading data: {:?}", e);
                                            break;
                                        }
                                    }
                                }

                                // Return media not present error
                                let csw = CommandStatusWrapper {
                                    tag: cbw.tag,
                                    data_residue: 0, // We consumed all the data
                                    status: CommandStatus::Failed,
                                };

                                let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                                csw.to_buffer(&mut csw_buffer).unwrap();
                                self.write_ep.write(&csw_buffer).await.unwrap();
                            }
                        } else {
                            debug!("Write10 - No storage");

                            // We still need to consume the data
                            let block_size = 512;
                            let total_size = block_size * cmd.transfer_blocks as usize;
                            let mut remaining = total_size;

                            // Read and discard the data
                            while remaining > 0 {
                                let to_read = min(remaining, 64);
                                let mut buffer = [0u8; 64];
                                match self.read_ep.read(&mut buffer[..to_read]).await {
                                    Ok(read_size) => {
                                        remaining -= read_size;
                                    }
                                    Err(e) => {
                                        error!("Error reading data: {:?}", e);
                                        break;
                                    }
                                }
                            }

                            // Return error
                            let csw = CommandStatusWrapper {
                                tag: cbw.tag,
                                data_residue: 0, // We consumed all the data
                                status: CommandStatus::Failed,
                            };

                            let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                            csw.to_buffer(&mut csw_buffer).unwrap();
                            self.write_ep.write(&csw_buffer).await.unwrap();
                        }
                    }
                    _ => {
                        debug!("ERROR: Unhandled command");

                        // Return failed status for unhandled commands
                        let csw = CommandStatusWrapper {
                            tag: cbw.tag,
                            data_residue: cbw.data_transfer_length,
                            status: CommandStatus::Failed,
                        };

                        let mut csw_buffer = [0u8; CommandStatusWrapper::SIZE];
                        csw.to_buffer(&mut csw_buffer).unwrap();
                        self.write_ep.write(&csw_buffer).await.unwrap();
                    }
                };

                // Parse SCSI Command

                //     x if x == ScsiCommand::ReadFormatCapacities as u8 => {
                //         debug!("Read Format Capacities");
                //         // Read Format Capacities data. resp fixed data
                //         let read_format_capacities_data = ReadFormatCapacitiesData::new(
                //             self.config.num_blocks as u32,
                //             self.config.block_size as u32,
                //         );

                //         let mut write_data = [0u8; READ_FORMAT_CAPACITIES_DATA_SIZE];
                //         read_format_capacities_data.prepare_to_buf(&mut write_data);
                //         Self::handle_response_single(
                //             write_ep,
                //             CommandBlockStatus::CommandPassed,
                //             Some(&write_data),
                //             &cbw_packet,
                //             &mut csw_packet,
                //         )
                //         .await
                //     }
                //     x if x == ScsiCommand::ReadCapacity as u8 => {
                //         debug!("Read Capacity");
                //         // Read Capacity data. resp fixed data
                //         let read_capacity_data = ReadCapacityData::new(
                //             (self.config.num_blocks - 1) as u32,
                //             self.config.block_size as u32,
                //         );

                //         let mut write_data = [0u8; READ_CAPACITY_16_DATA_SIZE];
                //         read_capacity_data.prepare_to_buf(&mut write_data);
                //         Self::handle_response_single(
                //             write_ep,
                //             CommandBlockStatus::CommandPassed,
                //             Some(&write_data),
                //             &cbw_packet,
                //             &mut csw_packet,
                //         )
                //         .await
                //     }
                //     x if x == ScsiCommand::ModeSense6 as u8 => {
                //         debug!("Mode Sense 6");
                //         // Mode Sense 6 data. resp fixed data
                //         let mode_sense_data = ModeSense6Data::new();

                //         let mut write_data = [0u8; MODE_SENSE_6_DATA_SIZE];
                //         mode_sense_data.prepare_to_buf(&mut write_data);
                //         Self::handle_response_single(
                //             write_ep,
                //             CommandBlockStatus::CommandPassed,
                //             Some(&write_data),
                //             &cbw_packet,
                //             &mut csw_packet,
                //         )
                //         .await
                //     }
                //     x if x == ScsiCommand::RequestSense as u8 => {
                //         // Error reporting
                //         if latest_sense_data.is_none() {
                //             latest_sense_data = Some(RequestSenseData::from(
                //                 SenseKey::NoSense,
                //                 AdditionalSenseCodeType::NoAdditionalSenseInformation,
                //             ));
                //         }
                //         debug!("Request Sense Data: {:#x}", latest_sense_data.unwrap());

                //         let mut write_data = [0u8; REQUEST_SENSE_DATA_SIZE];
                //         latest_sense_data.unwrap().prepare_to_buf(&mut write_data);
                //         Self::handle_response_single(
                //             write_ep,
                //             CommandBlockStatus::CommandPassed,
                //             Some(&write_data),
                //             &cbw_packet,
                //             &mut csw_packet,
                //         )
                //         .await
                //     }
                //     x if x == ScsiCommand::Read10 as u8 => {
                //         // Read 10 data. resp variable data
                //         let read10_data = Read10Command::from_data(scsi_commands);
                //         debug!("Read 10 Data: {:#x}", read10_data);
                //         let transfer_length = read10_data.transfer_length as usize;

                //         // TODO: channelに空きがある場合transfer_length分のRequest投げるTaskと、Responseを受け取るTaskのjoinにする
                //         for transfer_index in 0..transfer_length {
                //             let lba = read10_data.lba as usize + transfer_index;
                //             let req_tag = MscReqTag::new(cbw_packet.tag, transfer_index as u32);
                //             let req = StorageRequest::read(req_tag, lba);

                //             self.data_request_sender.send(req).await;
                //             let resp = self.data_response_receiver.receive().await;

                //             // Read処理中にRead以外の応答が来た場合は実装不具合
                //             if resp.message_id != StorageMsgId::Read {
                //                 crate::unreachable!("Invalid Response: {:#x}", resp);
                //             }
                //             // Check if the response is valid
                //             if (req_tag != resp.req_tag) {
                //                 error!("Invalid Response: {:#x}", resp);
                //                 latest_sense_data = Some(RequestSenseData::from(
                //                     SenseKey::HardwareError,
                //                     AdditionalSenseCodeType::HardwareErrorEmbeddedSoftware,
                //                 ));
                //             }
                //             // Check if there is an error
                //             if let Some(error) = resp.meta_data {
                //                 error!("Invalid Response: {:#x}", resp);
                //                 latest_sense_data =
                //                     Some(RequestSenseData::from_data_request_error(error));
                //             }

                //             // transfer read data
                //             let read_data = resp.data.as_ref();
                //             for packet_i in 0..USB_PACKET_COUNT_PER_LOGICAL_BLOCK {
                //                 let start_index = (packet_i * USB_MAX_PACKET_SIZE);
                //                 let end_index = ((packet_i + 1) * USB_MAX_PACKET_SIZE);
                //                 // 範囲がUSB_BLOCK_SIZEを超えないように修正
                //                 let end_index = end_index.min(USB_LOGICAL_BLOCK_SIZE);

                //                 // データを取り出して応答
                //                 let packet_data = &read_data[start_index..end_index];
                //                 debug!(
                //                     "Send Read Data (LBA: {:#x}, TransferIndex: {:#x}, PacketIndex: {:#x}): {:#x}",
                //                     lba, transfer_index, packet_i, packet_data
                //                 );
                //                 let Ok(write_resp) = write_ep.write(packet_data).await else {
                //                     error!("Write EP Error (Read 10)");
                //                     phase_error_tag = Some(cbw_packet.tag);
                //                     latest_sense_data = Some(RequestSenseData::from(
                //                         SenseKey::IllegalRequest,
                //                         AdditionalSenseCodeType::IllegalRequestInvalidCommand,
                //                     ));
                //                     break;
                //                 };
                //             }
                //         }

                //         // CSW 応答
                //         csw_packet.status =
                //             CommandBlockStatus::from_bool(latest_sense_data.is_none());
                //         let transfer_bytes = transfer_length * self.config.block_size;
                //         if transfer_bytes < cbw_packet.data_transfer_length as usize {
                //             csw_packet.data_residue =
                //                 (cbw_packet.data_transfer_length as usize - transfer_bytes) as u32;
                //         }
                //         let csw_data = csw_packet.to_data();
                //         debug!("Send CSW: {:#x}", csw_packet);
                //         write_ep.write(&csw_data).await
                //     }
                //     x if x == ScsiCommand::Write10 as u8 => {
                //         // Write 10 data. resp variable data
                //         let write10_data = Write10Command::from_data(scsi_commands);
                //         debug!("Write 10 Data: {:#x}", write10_data);
                //         let transfer_length = write10_data.transfer_length as usize;

                //         for transfer_index in 0..transfer_length {
                //             let lba = write10_data.lba as usize + transfer_index;
                //             // packet size分のデータを受け取る
                //             let req_tag = MscReqTag::new(cbw_packet.tag, transfer_index as u32);
                //             let mut req =
                //                 StorageRequest::write(req_tag, lba, [0u8; USB_LOGICAL_BLOCK_SIZE]);
                //             for packet_i in 0..USB_PACKET_COUNT_PER_LOGICAL_BLOCK {
                //                 let start_index = (packet_i * USB_MAX_PACKET_SIZE);
                //                 let end_index = ((packet_i + 1) * USB_MAX_PACKET_SIZE);
                //                 // 範囲がUSB_BLOCK_SIZEを超えないように修正
                //                 let end_index = end_index.min(USB_LOGICAL_BLOCK_SIZE);

                //                 // データを受け取る
                //                 let Ok(read_resp) =
                //                     read_ep.read(&mut req.data[start_index..end_index]).await
                //                 else {
                //                     error!("Read EP Error (Write 10)");
                //                     phase_error_tag = Some(cbw_packet.tag);
                //                     latest_sense_data = Some(RequestSenseData::from(
                //                         SenseKey::IllegalRequest,
                //                         AdditionalSenseCodeType::IllegalRequestInvalidCommand,
                //                     ));
                //                     break;
                //                 };
                //             }

                //             debug!("Send DataRequest: {:#x}", req);
                //             self.data_request_sender.send(req).await;

                //             let resp = self.data_response_receiver.receive().await;
                //             debug!("Receive DataResponse: {:#x}", resp);

                //             // Write処理中にWrite以外の応答が来た場合は実装不具合
                //             if resp.message_id != StorageMsgId::Write {
                //                 crate::unreachable!("Invalid Response: {:#x}", resp);
                //             }

                //             // Check if the response is valid
                //             if (req_tag != resp.req_tag) {
                //                 error!("Invalid Response: {:#x}", resp);
                //                 latest_sense_data = Some(RequestSenseData::from(
                //                     SenseKey::HardwareError,
                //                     AdditionalSenseCodeType::HardwareErrorEmbeddedSoftware,
                //                 ));
                //             }
                //             // Check if there is an error
                //             if let Some(error) = resp.meta_data {
                //                 error!("Invalid Response: {:#x}", resp);
                //                 latest_sense_data =
                //                     Some(RequestSenseData::from_data_request_error(error));
                //             }
                //         }

                //         // CSW 応答
                //         csw_packet.status =
                //             CommandBlockStatus::from_bool(latest_sense_data.is_none());
                //         let transfer_bytes = transfer_length * self.config.block_size;
                //         if transfer_bytes < cbw_packet.data_transfer_length as usize {
                //             csw_packet.data_residue =
                //                 (cbw_packet.data_transfer_length as usize - transfer_bytes) as u32;
                //         }
                //         let csw_data = csw_packet.to_data();
                //         write_ep.write(&csw_data).await
                //     }
                //     x if x == ScsiCommand::PreventAllowMediumRemoval as u8 => {
                //         debug!("Prevent/Allow Medium Removal");
                //         // カードの抜き差しを許可する
                //         Self::handle_response_single(
                //             write_ep,
                //             CommandBlockStatus::CommandPassed,
                //             None,
                //             &cbw_packet,
                //             &mut csw_packet,
                //         )
                //         .await
                //     }
                //     _ => {
                //         error!("Unsupported Command: {:#x}", scsi_command);
                //         // save latest sense data
                //         latest_sense_data = Some(RequestSenseData::from(
                //             SenseKey::IllegalRequest,
                //             AdditionalSenseCodeType::IllegalRequestInvalidCommand,
                //         ));

                //         Self::handle_response_single(
                //             write_ep,
                //             CommandBlockStatus::CommandFailed,
                //             None,
                //             &cbw_packet,
                //             &mut csw_packet,
                //         )
                //         .await
                //     }
                // };

                // Phase Error時の対応
                // if let Err(e) = send_resp_status {
                //     error!("Send Response Error: {:?}", e);
                //     // Phase Error時の対応用にtagを保持
                //     phase_error_tag = Some(cbw_packet.tag);
                //     break;
                // }
            }

            // // CSW で Phase Error を返す
            // if let Some(tag) = phase_error_tag {
            //     error!("Phase Error Tag: {:#x}", tag);
            //     let mut csw_packet = CommandStatusWrapperPacket::new();
            //     csw_packet.tag = tag;
            //     csw_packet.data_residue = 0;
            //     csw_packet.status = CommandBlockStatus::PhaseError;
            //     let csw_data = csw_packet.to_data();
            //     // 失敗してもハンドリング無理
            //     write_ep.write(&csw_data).await;
            // }
            debug!("Disconnected");
        }
    }
}

/*
*
*
MassStorageClass<
    'static,
    Driver<'static, USB>,
    SdCard<
        'static,
        NoopRawMutex,
        Spi<'static, SPI0, Async>,
        SharedSpiBusWithConfig<'static, NoopRawMutex, Spi<'static, SPI0, Async>>,
        Output<'static>,
    >,
>,
*/
#[embassy_executor::task]
pub async fn task(
    mut class: MassStorageClass<
        'static,
        embassy_rp::usb::Driver<'static, USB>,
        SdCard<
            'static,
            NoopRawMutex,
            Spi<'static, SPI0, Async>,
            SharedSpiBusWithConfig<'static, NoopRawMutex, Spi<'static, SPI0, Async>>,
            AsyncOutputPin<Output<'static>>,
        >,
    >,
) {
    class.run().await;
}
