use crate::{
    apps::tx::{TxController, TxMode},
    platform::{
        i2c_io_expander::{
            self,
            models::{pca9536::PCA9536, tcal9539::TCAL9539},
        },
        repl::{
            common::AckSignal,
            rpc::{RpcError, RpcResult},
            tx::{TxCommand, TxReceiver},
        },
    },
};
use alloc::string::String;
use defmt::{debug, warn};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_rp::{
    i2c,
    peripherals::{
        DMA_CH5, I2C0, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25, PIN_26,
        PIN_27, PIN_28, PIO1,
    },
    Peri,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[embassy_executor::task]
pub async fn tx_task(
    tx_rx: TxReceiver,
    tx_ack: &'static AckSignal,
    pio: Peri<'static, PIO1>,
    dma: Peri<'static, DMA_CH5>,
    l_z0: Peri<'static, PIN_18>,
    l_v2: Peri<'static, PIN_19>,
    l_v1: Peri<'static, PIN_20>,
    l_v0: Peri<'static, PIN_21>,
    h_z0: Peri<'static, PIN_22>,
    nil_0: Peri<'static, PIN_23>,
    nil_1: Peri<'static, PIN_24>,
    led: Peri<'static, PIN_25>,
    h_v2: Peri<'static, PIN_26>,
    h_v1: Peri<'static, PIN_27>,
    h_v0: Peri<'static, PIN_28>,
    tx_connect: i2c_io_expander::pin::Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        TCAL9539,
    >,
    tx_enable: i2c_io_expander::pin::Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        TCAL9539,
    >,
    pwr_injector: i2c_io_expander::pin::Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        PCA9536,
    >,
) -> ! {
    warn!("IN THE TX TASK!");

    let mut ctrl = unsafe {
        TxController::new(
            TxMode::Can,
            dma,
            pio,
            l_z0,
            l_v2,
            l_v1,
            l_v0,
            h_z0,
            nil_0,
            nil_1,
            led,
            h_v2,
            h_v1,
            h_v0,
            tx_connect,
            tx_enable,
            pwr_injector,
        )
    }
    .await
    .unwrap();

    // Logic table
    // H_V0, H_V1, H_V2, OUT
    //    0,    0,    0,   0
    //    0,    0,    1, 2.5
    //    0,    1,    0, 1.5
    //    0,    1,    1, 3.5
    //    1,    0,    0,   1
    //    1,    0,    1,   3
    //    1,    1,    0,   2
    //    1,    1,    1,   4
    //
    // Bit order
    // H_V0 H_V1 H_V2 H_Z0 L_V0 L_V1 L_V2 L_Z0

    loop {
        match tx_rx.receive().await {
            TxCommand::IsEnabled => {
                debug!("Tx IsEnabled");
                tx_ack.signal(Ok(RpcResult::TxIsEnabled(ctrl.is_enabled())))
            }
            TxCommand::EnableDisable(enabled) => {
                debug!("Tx EnableDisable {}", enabled);

                if enabled {
                    ctrl.enable().await;
                } else {
                    ctrl.disable().await;
                }

                tx_ack.signal(Ok(RpcResult::TxEnableDisable));
            }
            TxCommand::GetBaud => {
                debug!("Tx GetBaud");
                tx_ack.signal(Ok(RpcResult::TxGetBaud(ctrl.get_baud())));
            }
            TxCommand::SetBaud(baud) => {
                debug!("Tx SetBaud {}", baud);

                if let Err(divider) = ctrl.set_baud(baud) {
                    tx_ack.signal(Err(RpcError::ErrorArithmetic(defmt::format!(
                        "Invalid clock divider: {}",
                        divider
                    ))));
                }

                let outcome = ctrl
                    .set_baud(baud)
                    .map_err(|err| {
                        RpcError::ErrorArithmetic(defmt::format!("Invalid clock divider: {}", err))
                    })
                    .map(|_| RpcResult::TxSetBaud);
                tx_ack.signal(outcome);
            }
            TxCommand::GetMode => {
                debug!("Tx GetMode {:?}", ctrl.mode());
                tx_ack.signal(Ok(RpcResult::TxGetMode(ctrl.mode())));
            }
            TxCommand::SetMode(mode) => {
                debug!("Tx SetMode {:?}", mode);
                // TODO: Broken.
                ctrl.set_mode(mode).await;
                tx_ack.signal(Ok(RpcResult::TxSetMode));
            }
            TxCommand::Send(words) => {
                debug!("Tx Send {:?}", words.mode());

                if ctrl.is_enabled() {
                    ctrl.send(words).await;
                    tx_ack.signal(Ok(RpcResult::TxSend));
                } else {
                    tx_ack.signal(Err(RpcError::ErrorDataRace(String::from(
                        "tx is not enabled!",
                    ))));
                }
            }
        }
    }
}
