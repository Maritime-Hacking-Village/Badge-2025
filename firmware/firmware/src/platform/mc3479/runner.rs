use super::data::Data;
use crate::platform::interrupt_i2c::update_state::UpdateState;
use alloc::sync::Arc;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex},
    mutex::Mutex,
    signal, watch,
};
use embedded_hal_async::i2c::I2c;

pub const SHAKE_MTU: usize = 2;

pub type ShakeWatch = watch::Watch<CriticalSectionRawMutex, bool, SHAKE_MTU>;
pub type ShakeSender = watch::Sender<'static, CriticalSectionRawMutex, bool, SHAKE_MTU>;
pub type ShakeReceiver = watch::Receiver<'static, CriticalSectionRawMutex, bool, SHAKE_MTU>;

pub type ShakeSignal = signal::Signal<CriticalSectionRawMutex, ()>;

#[macro_export]
macro_rules! make_shake_channels {
    () => {{
        use crate::platform::mc3479::runner::{ShakeSignal, ShakeWatch};
        use embassy_sync::lazy_lock::LazyLock;

        static SHAKE_WATCH: LazyLock<ShakeWatch> = LazyLock::new(|| ShakeWatch::new_with(false));
        static SHAKE_SIGNAL_READY: LazyLock<ShakeSignal> = LazyLock::new(|| ShakeSignal::new());
        static SHAKE_SIGNAL_DONE: LazyLock<ShakeSignal> = LazyLock::new(|| ShakeSignal::new());

        (
            SHAKE_WATCH.get(),
            SHAKE_SIGNAL_READY.get(),
            SHAKE_SIGNAL_DONE.get(),
        )
    }};
}

pub struct Runner<M: RawMutex + 'static, D: I2c + 'static> {
    dev: I2cDevice<'static, M, D>,
    data: Arc<Mutex<M, Data>>,
    shake_tx: ShakeSender,
}

impl<M: RawMutex + 'static, D: I2c + 'static> Runner<M, D> {
    pub fn new(
        dev: I2cDevice<'static, M, D>,
        data: Arc<Mutex<M, Data>>,
        shake_tx: ShakeSender,
    ) -> Self {
        Self {
            dev,
            data,
            shake_tx,
        }
    }
}

impl<M: RawMutex + 'static, D: I2c + 'static> UpdateState for Runner<M, D> {
    async fn update(&mut self) -> () {
        // let reg = [0x17];
        // let mut data = [0u8, 0u8];
        // self.dev.write_read(0x4c, &reg, &mut data).await.unwrap();
        // let chip_id = data[1];

        // error!("CHIP ID: {:x}", chip_id);

        // read device status
        let reg = [0x05];
        let mut data = [0u8];
        self.dev.write_read(0x4c, &reg, &mut data).await.unwrap();
        let device_status = data[0];

        // read data
        let reg = [0x0D];
        let mut data = [0u8; 6];
        self.dev.write_read(0x4c, &reg, &mut data).await.unwrap();

        // read status
        let reg = [0x13];
        let mut statuses = [0u8, 0u8];
        self.dev
            .write_read(0x4c, &reg, &mut statuses)
            .await
            .unwrap();

        // clear interrupts
        let clear_int = [0x14u8, 0x00u8]; // send back interrupt status
        self.dev.write(0x4c, &clear_int).await.unwrap();

        // Debug to check to make sure they cleared correctly.
        let reg = [0x14];
        let mut cleared_interrupt_status = [0u8];
        self.dev
            .write_read(0x4c, &reg, &mut cleared_interrupt_status)
            .await
            .unwrap();

        let mut data_guard = self.data.lock().await;
        data_guard.x = (data[1] as i16) << 8 | (data[0] as i16);
        data_guard.y = (data[3] as i16) << 8 | (data[2] as i16);
        data_guard.z = (data[5] as i16) << 8 | (data[4] as i16);
        data_guard.status = statuses[0].into();
        data_guard.interrupt_status = statuses[1].into();

        // Puke.
        if data_guard.status.anym || data_guard.status.shake {
            self.shake_tx.send_if_modified(|maybe_shooketh| {
                if let Some(shooketh) = maybe_shooketh {
                    if *shooketh {
                        return false;
                    }
                }

                *maybe_shooketh = Some(true);
                true
            });
        }
    }
}
