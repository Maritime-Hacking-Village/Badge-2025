pub mod can;
pub mod modbus;
pub mod nmea0183;

use crate::{
    apps::rx::can::{PioCanRx, PioCanRxProgram},
    platform::{i2c_io_expander, i2c_io_expander::models::pca9536::PCA9536, irqs::Irqs},
};
use defmt::{error, Format};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_rp::{
    i2c,
    peripherals::{DMA_CH4, I2C0, PIN_9, PIO2, UART1},
    pio::Pio,
    uart::{Async, Config, UartRx},
    Peri,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum RxError {
    ClockDividerTooLarge,
    ClockDividerTooSmall,
}

impl core::fmt::Display for RxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait SerialParser {
    type Word;
    type Message;
    type Error;

    /// Adds the given byte to the parser's state machine and returns the resulting message, or parsing error, or None if the message is not ready.
    fn parse_word(&mut self, word: Self::Word) -> Option<Result<Self::Message, Self::Error>>;

    /// Resets parser state.
    fn reset(&mut self);

    /// The MTU of the given protocol. To be used for specifying buffer size.
    fn mtu() -> usize;

    fn default_baud() -> u32;
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum RxMode {
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

pub enum RxState {
    Uart(UartRx<'static, Async>, u32),
    Pio(PioCanRx<'static, PIO2, 0>),
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum RxWord {
    Nmea0183(<nmea0183::Parser as SerialParser>::Word),
    Modbus(u8),
    Can(<can::Parser as SerialParser>::Word),
}

pub struct RxController {
    uart: Peri<'static, UART1>,
    pio: Peri<'static, PIO2>,
    dma: Peri<'static, DMA_CH4>,
    rx_pin: Peri<'static, PIN_9>,
    pwr_receiver: i2c_io_expander::pin::Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        PCA9536,
    >,
    mode: RxMode,
    state: RxState,
    enabled: bool,
}

impl RxController {
    pub async unsafe fn new(
        mode: RxMode,
        uart: Peri<'static, UART1>,
        pio: Peri<'static, PIO2>,
        dma: Peri<'static, DMA_CH4>,
        rx_pin: Peri<'static, PIN_9>,
        mut pwr_receiver: i2c_io_expander::pin::Pin<
            CriticalSectionRawMutex,
            I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
            PCA9536,
        >,
    ) -> Self {
        // Set up default disabled pin configuration.
        pwr_receiver.set_direction(false).await;
        pwr_receiver.set_output(true).await;

        let uart_cpy = uart.clone_unchecked();
        let pio_cpy = pio.clone_unchecked();
        let dma_cpy = dma.clone_unchecked();
        let rx_pin_cpy = rx_pin.clone_unchecked();

        let state = match mode {
            RxMode::Nmea0183 => {
                let mut cfg = Config::default();
                let baud = nmea0183::Parser::default_baud();
                cfg.baudrate = nmea0183::Parser::default_baud();
                RxState::Uart(UartRx::new(uart_cpy, rx_pin_cpy, Irqs, dma_cpy, cfg), baud)
            }
            RxMode::Modbus => {
                let mut cfg = Config::default();
                let baud = 9600;
                cfg.baudrate = baud; // TODO modbus::Parser::default_baud();
                RxState::Uart(UartRx::new(uart_cpy, rx_pin_cpy, Irqs, dma_cpy, cfg), baud)
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
            pwr_receiver,
        }
    }

    pub async fn enable(&mut self) {
        match &mut self.state {
            RxState::Uart(_, _) => {}
            RxState::Pio(pio_rx) => {
                pio_rx.enable();
            }
        }

        self.pwr_receiver.set_output(false).await;
        self.enabled = true;
    }

    pub async fn disable(&mut self) {
        match &mut self.state {
            RxState::Uart(_, _) => {}
            RxState::Pio(pio_rx) => {
                pio_rx.disable();
            }
        }

        self.pwr_receiver.set_output(true).await;
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_baud(&mut self) -> u32 {
        defmt::debug!("GET BAUD");
        match &self.state {
            RxState::Uart(_, baud) => *baud,
            RxState::Pio(pio) => pio.get_baud(),
        }
    }

    pub unsafe fn set_baud(&mut self, baud: u32) -> Result<(), RxError> {
        let mut new_uart: Option<UartRx<'static, Async>> = None;

        match &mut self.state {
            RxState::Uart(_, _) => {
                let mut cfg = Config::default();
                cfg.baudrate = baud;
                new_uart = Some(UartRx::new(
                    self.uart.clone_unchecked(),
                    self.rx_pin.clone_unchecked(),
                    Irqs,
                    self.dma.clone_unchecked(),
                    cfg,
                ));
            }
            RxState::Pio(pio) => pio.set_baud(baud)?,
        }

        // TODO: This is pretty janky.
        if let Some(new_uart) = new_uart {
            self.state = RxState::Uart(new_uart, baud);
        }

        Ok(())
    }

    pub async unsafe fn set_mode(&mut self, mode: RxMode) {
        if self.mode == mode {
            return;
        }

        let enabled = self.enabled;

        if enabled {
            self.disable().await;
        }

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
                        let baud = nmea0183::Parser::default_baud();
                        cfg.baudrate = baud;
                        self.state = RxState::Uart(
                            UartRx::new(
                                self.uart.clone_unchecked(),
                                self.rx_pin.clone_unchecked(),
                                Irqs,
                                self.dma.clone_unchecked(),
                                cfg,
                            ),
                            baud,
                        );
                    }
                    RxMode::Modbus => {
                        // Swap from PIO to UART.
                        let mut cfg = Config::default();
                        let baud = 9600;
                        cfg.baudrate = baud; // TODO modbus::Parser::default_baud();
                        self.state = RxState::Uart(
                            UartRx::new(
                                self.uart.clone_unchecked(),
                                self.rx_pin.clone_unchecked(),
                                Irqs,
                                self.dma.clone_unchecked(),
                                cfg,
                            ),
                            baud,
                        );
                    }
                    RxMode::Can => {
                        // Don't need to swap controllers.
                    }
                }
            }
        }

        self.mode = mode;

        if enabled {
            self.enable().await;
        }
    }

    pub fn mode(&self) -> RxMode {
        self.mode
    }

    // TODO: Custom error type.
    pub async fn read_word(&mut self) -> Option<RxWord> {
        match &mut self.state {
            RxState::Uart(uart_rx, _) => {
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
