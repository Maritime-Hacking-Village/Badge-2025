use crate::platform::{
    locking_spi_bus::LockingSpiBus,
    msc::scsi::cmd::{Read10, ReadCapacityResponse, Write10},
    sdmmc::spi::{SdCard, SdCardBus},
    set_frequency::SetFrequency,
};
use alloc::vec::Vec;
use core::iter;
use defmt::error;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embedded_hal_async::{digital::OutputPin, spi::SpiBus};

/// A trait for storage devices to be used with MSC
pub trait MscStorage {
    /// Returns the capacity information for the storage device
    fn get_capacity(&self) -> ReadCapacityResponse;

    /// Read blocks from the storage device
    async fn read_blocks(&mut self, cmd: &Read10) -> Result<Vec<[u8; 512]>, ()>;

    /// Write blocks to the storage device
    async fn write_blocks(&mut self, cmd: &Write10, data: &[[u8; 512]]) -> Result<(), ()>;

    /// Check if the storage device is present
    fn is_present(&self) -> bool;
}

/// Implementation for SD card storage
impl<'a, M, Braw, B, CS> MscStorage for SdCard<'a, M, Braw, B, CS>
where
    M: RawMutex + 'a,
    Braw: SdCardBus + SpiBus + 'a,
    B: LockingSpiBus<'a, M, Braw> + SetFrequency,
    CS: OutputPin,
{
    fn get_capacity(&self) -> ReadCapacityResponse {
        let blocks: u64 = self.num_blocks().into();
        ReadCapacityResponse {
            logical_block_address: (blocks - 1) as u32,
            block_length: 512,
        }
    }

    async fn read_blocks(&mut self, cmd: &Read10) -> Result<Vec<[u8; 512]>, ()> {
        let mut blocks = alloc::vec![[0u8; 512]; cmd.transfer_blocks as usize];

        match self.read(cmd.block_address as u32, blocks.iter_mut()).await {
            Ok(_) => {
                return Ok(blocks);
            }
            Err(_) => {
                error!("SD read error");
                return Err(());
            }
        }
    }

    async fn write_blocks(&mut self, cmd: &Write10, data: &[[u8; 512]]) -> Result<(), ()> {
        for (i, block) in data.iter().enumerate() {
            // Create iterator with a single block
            let block_iter = iter::once(block);

            // Write to SD card
            match self.write(cmd.block_address + i as u32, block_iter).await {
                Ok(_) => {}
                Err(_) => {
                    error!("SD write error");
                    return Err(());
                }
            }
        }

        Ok(())
    }

    fn is_present(&self) -> bool {
        // SD card initialized successfully, so it's present
        true
    }
}

/// A placeholder implementation when no storage is available
/// This always returns "not ready" errors to properly inform the host
pub struct NoStorage;

impl NoStorage {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MscStorage for NoStorage {
    fn get_capacity(&self) -> ReadCapacityResponse {
        // Return minimal capacity information
        // This won't actually be used as the MSC class will report media not present
        ReadCapacityResponse {
            logical_block_address: 0,
            block_length: 512,
        }
    }

    async fn read_blocks(&mut self, _cmd: &Read10) -> Result<Vec<[u8; 512]>, ()> {
        // Always fail with media not present
        Err(())
    }

    async fn write_blocks(&mut self, _cmd: &Write10, _data: &[[u8; 512]]) -> Result<(), ()> {
        // Always fail with media not present
        Err(())
    }

    fn is_present(&self) -> bool {
        // This implementation represents no storage
        false
    }
}
