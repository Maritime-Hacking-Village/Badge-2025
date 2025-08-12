pub mod can_pio;
pub mod can_spi;
pub mod inject;

use crate::{
    apps::tx::{
        can_pio::{PioCanTrx, PioCanTrxProgram},
        inject::{PioInjector, PioInjectorProgram},
    },
    platform::{
        i2c_io_expander::{
            self,
            models::{pca9536::PCA9536, tcal9539::TCAL9539},
        },
        irqs::Irqs,
    },
};
use alloc::vec::Vec;
use defmt::Format;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_rp::{
    dma::Channel,
    i2c,
    interrupt::typelevel::Binding,
    peripherals::{
        I2C0, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25, PIN_26, PIN_27,
        PIN_28,
    },
    pio::{Instance, Pio},
    Peri,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum TxError {
    ClockDividerTooLarge,
    ClockDividerTooSmall,
}

impl core::fmt::Display for TxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl core::error::Error for TxError {}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum TxMode {
    Inject,
    Can,
}

// pub enum TxState<'a, P: Instance> {
//     Inject(PioInjector<'a, P, 0>),
//     Can(PioCanTrx<'a, P, 1>),
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxWords {
    Inject(Vec<u8>),
    Can(Vec<u32>),
}

impl Format for TxWords {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{:?}", defmt::Debug2Format(self))
    }
}

impl TxWords {
    pub fn mode(&self) -> TxMode {
        match self {
            TxWords::Inject(_) => TxMode::Inject,
            TxWords::Can(_) => TxMode::Can,
        }
    }
}

pub struct TxController<'a, P: Instance> {
    mode: TxMode,
    enabled: bool,
    pio_inj: PioInjector<'a, P, 0>,
    pio_can: PioCanTrx<'a, P, 1>,
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
}

impl<'a, P: Instance> TxController<'a, P>
where
    Irqs: Binding<<P as Instance>::Interrupt, embassy_rp::pio::InterruptHandler<P>>,
{
    pub async unsafe fn new(
        mode: TxMode,
        dma: Peri<'a, impl Channel>,
        pio: Peri<'a, P>,
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
        mut tx_connect: i2c_io_expander::pin::Pin<
            CriticalSectionRawMutex,
            I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
            TCAL9539,
        >,
        mut tx_enable: i2c_io_expander::pin::Pin<
            CriticalSectionRawMutex,
            I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
            TCAL9539,
        >,
        mut pwr_injector: i2c_io_expander::pin::Pin<
            CriticalSectionRawMutex,
            I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
            PCA9536,
        >,
    ) -> Result<Self, TxError> {
        let Pio {
            mut common,
            sm0,
            sm1,
            ..
        } = Pio::new(pio, Irqs);
        let prg_inj = PioInjectorProgram::new(&mut common);
        let pio_inj = PioInjector::new(
            Self::default_baud(),
            &mut common,
            sm0,
            dma,
            l_z0.clone_unchecked(),
            l_v2.clone_unchecked(),
            l_v1.clone_unchecked(),
            l_v0.clone_unchecked(),
            h_z0.clone_unchecked(),
            nil_0,
            nil_1,
            led,
            h_v2.clone_unchecked(),
            h_v1.clone_unchecked(),
            h_v0.clone_unchecked(),
            &prg_inj,
        )?;
        let prg_can = PioCanTrxProgram::new(&mut common);
        let pio_can = PioCanTrx::new(
            Self::default_baud(),
            &mut common,
            sm1,
            l_z0,
            l_v2,
            l_v1,
            l_v0,
            h_z0,
            h_v2,
            h_v1,
            h_v0,
            &prg_can,
        )?;

        // Disabled by default
        tx_connect.set_direction(false).await;
        tx_connect.set_output(false).await;
        tx_enable.set_direction(false).await;
        tx_enable.set_output(false).await;
        pwr_injector.set_direction(false).await;
        pwr_injector.set_output(false).await;

        Ok(Self {
            mode,
            enabled: false,
            pio_inj,
            pio_can,
            tx_connect,
            tx_enable,
            pwr_injector,
        })
    }

    pub fn default_baud() -> u32 {
        250_000
    }

    pub fn mode(&self) -> TxMode {
        self.mode
    }

    pub async fn set_mode(&mut self, mode: TxMode) {
        if self.mode == mode {
            return;
        }

        let enabled = self.enabled;

        if enabled {
            self.disable().await;
        }

        self.mode = mode;

        if enabled {
            self.enable().await;
        }
    }

    pub fn get_baud(&self) -> u32 {
        match self.mode {
            TxMode::Inject => self.pio_inj.get_baud(),
            TxMode::Can => self.pio_can.get_baud(),
        }
    }

    pub fn set_baud(&mut self, baud: u32) -> Result<(), TxError> {
        self.pio_inj.set_baud(baud)?;
        self.pio_can.set_baud(baud)?;

        Ok(())
    }

    pub fn is_enabled(&mut self) -> bool {
        return self.enabled;
    }

    pub async fn enable(&mut self) {
        match self.mode {
            TxMode::Inject => self.pio_inj.enable(),
            TxMode::Can => {
                self.pio_can.enable();
            }
        }

        self.tx_connect.set_output(true).await;
        self.tx_enable.set_output(true).await;
        self.pwr_injector.set_output(true).await;
        self.enabled = true;
    }

    pub async fn disable(&mut self) {
        self.pio_inj.disable();
        self.pio_can.disable();
        self.tx_connect.set_output(false).await;
        self.tx_enable.set_output(false).await;
        self.pwr_injector.set_output(false).await;
        self.enabled = false;
    }

    pub async fn send(&mut self, words: TxWords) {
        assert_eq!(words.mode(), self.mode);

        match self.mode {
            TxMode::Inject => match words {
                TxWords::Inject(words) => {
                    self.pio_inj.write_bytes(&words).await;
                }
                TxWords::Can(_) => {
                    unreachable!()
                }
            },
            TxMode::Can => match words {
                TxWords::Inject(_) => {
                    unreachable!()
                }
                TxWords::Can(words) => {
                    for word in words {
                        self.pio_can.write_word(word).await;
                    }
                }
            },
        }
    }
}
