use crate::{
    apps::logging::Logger,
    platform::{multi_write::MultiWrite, repl::console::CONSOLE_MTU, usb_cdc_io::UsbCdcIo},
};

#[embassy_executor::task]
pub async fn log_task(mut logger: Logger<'static, MultiWrite<UsbCdcIo<'static>, CONSOLE_MTU>>) {
    logger.run().await
}
