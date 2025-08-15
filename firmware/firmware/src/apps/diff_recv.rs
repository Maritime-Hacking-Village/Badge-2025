use crate::{
    apps::rx::{
        can::{self, PioCanRx, PioCanRxProgram},
        nmea0183, SerialParser,
    },
    platform::{
        i2c_io_expander::{models::pca9536::PCA9536, pin::Pin},
        irqs::Irqs,
        repl::{
            common::AckSignal,
            rpc::RpcResult,
            rx::{RxCommand, RxReceiver},
        },
    },
};
use defmt::{debug, error, warn, Format};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_futures::select::{self, Either};
use embassy_rp::{
    i2c,
    peripherals::{DMA_CH4, I2C0, PIN_9, PIO2, UART1},
    pio::Pio,
    uart::{Async, Config, UartRx},
    Peri,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RxMode {
    Nmea0183,
    Modbus,
    Can,
}

impl From<RxWord> for RxMode {
    fn from(value: RxWord) -> RxMode {
        match value {
            RxWord::Nmea0183(_) => RxMode::Nmea0183,
            RxWord::Modbus(_) => RxMode::Modbus,
            RxWord::Can(_) => RxMode::Can,
        }
    }
}

pub(crate) enum RxState {
    Uart(UartRx<'static, Async>),
    Pio(PioCanRx<'static, PIO2, 0>),
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RxWord {
    Nmea0183(<nmea0183::Parser as SerialParser>::Word),
    Modbus(u8),
    Can(<can::Parser as SerialParser>::Word),
}

struct RxController {
    uart: Peri<'static, UART1>,
    pio: Peri<'static, PIO2>,
    dma: Peri<'static, DMA_CH4>,
    rx_pin: Peri<'static, PIN_9>,
    mode: RxMode,
    state: RxState,
    enabled: bool,
}

impl RxController {
    pub unsafe fn new(
        uart: Peri<'static, UART1>,
        pio: Peri<'static, PIO2>,
        dma: Peri<'static, DMA_CH4>,
        rx_pin: Peri<'static, PIN_9>,
        mode: RxMode,
    ) -> Self {
        let uart_cpy = uart.clone_unchecked();
        let pio_cpy = pio.clone_unchecked();
        let dma_cpy = dma.clone_unchecked();
        let rx_pin_cpy = rx_pin.clone_unchecked();

        let state = match mode {
            RxMode::Nmea0183 => {
                let mut cfg = Config::default();
                cfg.baudrate = nmea0183::Parser::default_baud();
                RxState::Uart(UartRx::new(uart_cpy, rx_pin_cpy, Irqs, dma_cpy, cfg))
            }
            RxMode::Modbus => {
                let mut cfg = Config::default();
                cfg.baudrate = 9600; // TODO modbus::Parser::default_baud();
                RxState::Uart(UartRx::new(uart_cpy, rx_pin_cpy, Irqs, dma_cpy, cfg))
            }
            RxMode::Can => {
                let Pio {
                    mut common,
                    sm0,
                    irq0,
                    irq1,
                    ..
                } = Pio::new(pio_cpy, Irqs);
                let can_rx_prog = PioCanRxProgram::new(&mut common);

                RxState::Pio(PioCanRx::new(
                    can::Parser::default_baud(),
                    &mut common,
                    sm0,
                    rx_pin_cpy,
                    &can_rx_prog,
                    irq0,
                    irq1,
                ))
            }
        };

        RxController {
            uart: uart,
            pio: pio,
            dma: dma,
            rx_pin: rx_pin,
            mode,
            state: state,
            enabled: false,
        }
    }

    pub fn enable(&mut self) {
        match &mut self.state {
            RxState::Uart(_) => {}
            RxState::Pio(pio_rx) => {
                pio_rx.enable();
            }
        }

        self.enabled = true;
    }

    pub fn disable(&mut self) {
        match &mut self.state {
            RxState::Uart(_) => {}
            RxState::Pio(pio_rx) => {
                pio_rx.disable();
            }
        }

        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub unsafe fn set_mode(&mut self, mode: RxMode) {
        match self.mode {
            RxMode::Nmea0183 | RxMode::Modbus => {
                match mode {
                    RxMode::Nmea0183 | RxMode::Modbus => {
                        // Don't need to swap controllers.
                    }
                    RxMode::Can => {
                        // Swap from UART to PIO.
                        let Pio {
                            mut common,
                            sm0,
                            irq0,
                            irq1,
                            ..
                        } = Pio::new(self.pio.clone_unchecked(), Irqs);
                        let can_rx_prog = PioCanRxProgram::new(&mut common);

                        // TODO: Need to make sure the old state gets dropped.
                        self.state = RxState::Pio(PioCanRx::new(
                            can::Parser::default_baud(),
                            &mut common,
                            sm0,
                            self.rx_pin.clone_unchecked(),
                            &can_rx_prog,
                            irq0,
                            irq1,
                        ))
                    }
                }
            }
            RxMode::Can => {
                match mode {
                    RxMode::Nmea0183 => {
                        // Swap from PIO to UART.
                        let mut cfg = Config::default();
                        cfg.baudrate = nmea0183::Parser::default_baud();
                        self.state = RxState::Uart(UartRx::new(
                            self.uart.clone_unchecked(),
                            self.rx_pin.clone_unchecked(),
                            Irqs,
                            self.dma.clone_unchecked(),
                            cfg,
                        ));
                    }
                    RxMode::Modbus => {
                        // Swap from PIO to UART.
                        let mut cfg = Config::default();
                        cfg.baudrate = 9600; // TODO modbus::Parser::default_baud();
                        self.state = RxState::Uart(UartRx::new(
                            self.uart.clone_unchecked(),
                            self.rx_pin.clone_unchecked(),
                            Irqs,
                            self.dma.clone_unchecked(),
                            cfg,
                        ));
                    }
                    RxMode::Can => {
                        // Don't need to swap controllers.
                    }
                }
            }
        }

        self.mode = mode;
    }

    pub fn mode(&self) -> RxMode {
        self.mode
    }

    // TODO: Custom error type.
    pub async fn read_word(&mut self) -> Option<RxWord> {
        match &mut self.state {
            RxState::Uart(uart_rx) => {
                let mut buf = [0_u8; 1];

                match uart_rx.read(&mut buf).await {
                    Ok(_) => match self.mode {
                        RxMode::Nmea0183 => return Some(RxWord::Nmea0183(buf[0])),
                        RxMode::Modbus => return Some(RxWord::Modbus(buf[0])),
                        RxMode::Can => {
                            unreachable!()
                        }
                    },
                    Err(err) => {
                        error!("UART error: {:?}", err);
                        return None;
                    }
                }
            }
            RxState::Pio(pio_rx) => {
                return Some(RxWord::Can(pio_rx.read_word().await));
            }
        }
    }
}

#[embassy_executor::task]
pub async fn diff_rx_task(
    uart: Peri<'static, UART1>,
    pio: Peri<'static, PIO2>,
    rx_pin: Peri<'static, PIN_9>,
    dma: Peri<'static, DMA_CH4>,
    mut pwr_receiver: Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        PCA9536,
    >,
    rx_rx: RxReceiver,
    rx_ack: &'static AckSignal,
    // TODO: Message queue back to the REPL.
) -> ! {
    error!("In diff Rx task!");

    // TODO: Maybe move these pins into the controller.
    // Set up default pin configuration.
    // Enable differential receiver IC.
    pwr_receiver.set_direction(false).await;
    pwr_receiver.set_output(false).await;

    // TODO: Maybe put parsers in the controller
    let mut ctrl = unsafe { RxController::new(uart, pio, dma, rx_pin, RxMode::Can) };
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
                        todo!();
                    }
                    RxWord::Can(word) => match can_parser.parse_word(word) {
                        Some(Ok(msg)) => {}
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
                RxCommand::EnableDisable(enabled) => {
                    debug!("EnableDisable: {}", enabled);

                    if enabled {
                        ctrl.enable();
                    } else {
                        ctrl.disable();
                    }

                    pwr_receiver.set_output(!enabled).await;
                    rx_ack.signal(Ok(RpcResult::RxEnableDisable));
                }
                RxCommand::SetMode(mode) => {
                    debug!("SetMode: {:?}", mode);
                    unsafe { ctrl.set_mode(mode) };
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
