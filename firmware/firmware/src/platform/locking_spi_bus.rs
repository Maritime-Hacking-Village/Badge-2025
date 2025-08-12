use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::MutexGuard};

pub trait LockingSpiBus<'a, M: RawMutex + 'a, SPI: 'a> {
    async fn lock(&mut self) -> MutexGuard<'a, M, SPI>;
    // TODO: Maybe add try_lock.
}
