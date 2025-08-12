use crate::platform::{
    locking_spi_bus::LockingSpiBus,
    sdmmc::spi::{SdCard, SdCardBus},
    set_frequency::SetFrequency,
};
use alloc::format;
use core::slice;
use defmt::debug;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embedded_hal_async::{digital::OutputPin, spi::SpiBus};
use mbr_nostd::{MasterBootRecord, PartitionTable};

pub async fn init<'a, M, Braw, B, CS>(sd: &mut SdCard<'a, M, Braw, B, CS>) -> Result<(), ()>
where
    M: RawMutex + 'a,
    Braw: SdCardBus + SpiBus + 'a,
    B: LockingSpiBus<'a, M, Braw> + SetFrequency,
    CS: OutputPin,
{
    debug!("will initialize");
    sd.init().await.map_err(|_| ())?;
    debug!("initialized");
    let csd = sd.read_csd().await.map_err(|_| ())?;

    let size: u64 = csd.num_blocks().into();
    debug!("Size {}", size * 512);

    let mut buffer = [0u8; 512];
    sd.read(0, slice::from_mut(&mut buffer).iter_mut())
        .await
        .map_err(|_| ())?;
    let mbr = MasterBootRecord::from_bytes(&buffer).map_err(|_| ())?;
    for partition in mbr.partition_table_entries().iter() {
        let s = format!("{:?}", partition);
        debug!("{}", s.as_str());
    }

    Ok(())
}
