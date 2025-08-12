use crate::{
    apps::rx::{
        can::{self},
        nmea0183, RxController, RxMode, RxWord, SerialParser,
    },
    platform::{
        i2c_io_expander::{models::pca9536::PCA9536, pin::Pin},
        repl::{
            common::AckSignal,
            rpc::{RpcError, RpcResult},
            rx::{RxBuffer, RxCommand, RxReceiver},
        },
    },
};
use bitvec::{order::Msb0, vec::BitVec};
use defmt::{debug, error, warn};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_futures::select::{self, Either};
use embassy_rp::{
    i2c,
    peripherals::{DMA_CH4, I2C0, PIN_9, PIO2, UART1},
    Peri,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[embassy_executor::task]
pub async fn rx_task(
    rx_rx: RxReceiver,
    rx_ack: &'static AckSignal,
    rx_buf: &'static RxBuffer,
    uart: Peri<'static, UART1>,
    pio: Peri<'static, PIO2>,
    rx_pin: Peri<'static, PIN_9>,
    dma: Peri<'static, DMA_CH4>,
    pwr_receiver: Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        PCA9536,
    >,
    // TODO: Message queue back to the REPL.
) -> ! {
    // TODO: Maybe put parsers in the controller
    let mut ctrl =
        unsafe { RxController::new(RxMode::Nmea0183, uart, pio, dma, rx_pin, pwr_receiver).await };
    let mut nmea0183_parser = nmea0183::Parser::new();
    let mut can_parser = can::Parser::new();

    loop {
        match select::select(ctrl.read_word(), rx_rx.receive()).await {
            Either::First(Some(word)) => {
                assert_eq!(RxMode::from(word), ctrl.mode());

                match word {
                    RxWord::Nmea0183(word) => {
                        match nmea0183_parser.parse_word(word) {
                            Some(Ok((sof, message, chksum))) => {
                                warn!(
                                    "Got NMEA-0183 message: {}{}*{:02X}",
                                    sof as char,
                                    message.as_str(),
                                    chksum
                                );
                                let mut blob: BitVec<u8, Msb0> = BitVec::from_element(sof);
                                blob.extend_from_raw_slice(message.as_bytes());
                                blob.extend_from_raw_slice(&[(chksum >> 8) as u8]);
                                blob.extend_from_raw_slice(&[(chksum & 0xFF) as u8]);
                                rx_buf.signal(blob);
                            }
                            Some(Err(err)) => {
                                error!("Error parsing NMEA-0183 message: {}", err);
                            }
                            None => {
                                // Not enough data for parsing.
                            }
                        }
                    }
                    RxWord::Modbus(word) => {
                        // TODO
                    }
                    RxWord::Can(word) => match can_parser.parse_word(word) {
                        Some(Ok(msg)) => {
                            warn!("Got CAN message!");
                            rx_buf.signal(msg);
                        }
                        Some(Err(err)) => {
                            error!("Error parsing CAN message: {}", err);
                        }
                        None => {
                            // Not enough data for parsing.
                        }
                    },
                }
            }
            Either::First(None) => {
                warn!("Got None back from Rx read word!");
            }
            Either::Second(cmd) => match cmd {
                RxCommand::IsEnabled => {
                    debug!("Rx IsEnabled");
                    rx_ack.signal(Ok(RpcResult::RxIsEnabled(ctrl.is_enabled())))
                }
                RxCommand::EnableDisable(enabled) => {
                    debug!("EnableDisable: {}", enabled);

                    if enabled {
                        ctrl.enable().await;
                    } else {
                        ctrl.disable().await;
                    }

                    rx_buf.reset();
                    rx_ack.signal(Ok(RpcResult::RxEnableDisable));
                }
                RxCommand::GetBaud => {
                    debug!("GetBaud");
                    rx_ack.signal(Ok(RpcResult::RxGetBaud(ctrl.get_baud())))
                }
                RxCommand::SetBaud(baud) => {
                    debug!("SetBaud {:?}", baud);
                    let outcome = unsafe { ctrl.set_baud(baud) }
                        .map_err(|err| {
                            RpcError::ErrorArithmetic(defmt::format!(
                                "Invalid clock divider: {}",
                                err
                            ))
                        })
                        .map(|_| RpcResult::RxSetBaud);
                    rx_buf.reset();
                    rx_ack.signal(outcome)
                }
                RxCommand::SetMode(mode) => {
                    debug!("SetMode: {:?}", mode);
                    unsafe { ctrl.set_mode(mode).await };
                    rx_buf.reset();
                    rx_ack.signal(Ok(RpcResult::RxSetMode));
                }
                RxCommand::GetMode => {
                    debug!("GetMode");
                    rx_ack.signal(Ok(RpcResult::RxGetMode(ctrl.mode())))
                }
            },
        }
    }
}
