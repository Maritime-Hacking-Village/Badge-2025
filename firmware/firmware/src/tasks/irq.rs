use crate::platform::{
    bq25895,
    i2c_io_expander::{self, models::tcal9539::TCAL9539},
    interrupt_i2c, mc3479,
};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_rp::{gpio::Input, i2c, peripherals::I2C0};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[embassy_executor::task]
pub async fn irq_task(
    mut runner: interrupt_i2c::Runner<
        Input<'static>,
        i2c_io_expander::Control<
            CriticalSectionRawMutex,
            I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
            TCAL9539,
            16,
        >,
        bq25895::runner::Runner<CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        mc3479::runner::Runner<CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
    >,
) -> ! {
    runner.run().await
}
