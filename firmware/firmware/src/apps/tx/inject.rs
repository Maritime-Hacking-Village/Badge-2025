use core::convert::Infallible;
use embassy_rp::{
    clocks::clk_sys_freq,
    dma::{AnyChannel, Channel},
    gpio::{Level, Output},
    peripherals::PIN_25,
    pio::{
        Common, Config, Direction, Instance, LoadedProgram, PioPin, ShiftDirection, StateMachine,
    },
    Peri,
};
use embedded_io_async::{ErrorType, Write};
use fixed::{traits::ToFixed, types::extra::U8, FixedU32};

use crate::apps::tx::TxError;

pub struct PioInjectorProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
}

impl<'d, PIO: Instance> PioInjectorProgram<'d, PIO> {
    /// Load the uart tx program into the given pio
    pub fn new(common: &mut Common<'d, PIO>) -> Self {
        // TODO: Is the behavior correct for the default?

        let prg = pio::pio_asm!(
            r#"
            set x, 0b10001      ; default: 0V, Hi-Z

            .wrap_target
                pull noblock        ; get data into the OSR
                in osr, 5           ; shift 5 bits from OSR into ISR
                out null, 5         ; flush 5 bits from OSR
                in null, 3          ; shift 0b000 bits into ISR
                in osr, 3           ; shift next 3 bits from OSR into ISR
                in null, 21         ; fill ISR with 21 zeros
                mov osr, isr        ; move ISR into OSR
                out pins, 11        ; write OSR to pins
            .wrap
        "#
        );
        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

pub struct PioInjector<'d, PIO: Instance, const SM: usize> {
    sm: StateMachine<'d, PIO, SM>,
    dma: Peri<'d, AnyChannel>,
    led: Output<'static>,
}

impl<'d, PIO: Instance, const SM: usize> PioInjector<'d, PIO, SM> {
    /// Configure a pio state machine to use the loaded tx program.
    pub fn new(
        baud: u32,
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        dma: Peri<'d, impl Channel>,
        l_z0: Peri<'d, impl PioPin>,
        l_v2: Peri<'d, impl PioPin>,
        l_v1: Peri<'d, impl PioPin>,
        l_v0: Peri<'d, impl PioPin>,
        h_z0: Peri<'d, impl PioPin>,
        nil_0: Peri<'d, impl PioPin>,
        nil_1: Peri<'d, impl PioPin>,
        led: Peri<'static, PIN_25>,
        h_v2: Peri<'d, impl PioPin>,
        h_v1: Peri<'d, impl PioPin>,
        h_v0: Peri<'d, impl PioPin>,
        program: &PioInjectorProgram<'d, PIO>,
    ) -> Result<Self, TxError> {
        let l_z0 = common.make_pio_pin(l_z0);
        let l_v2 = common.make_pio_pin(l_v2);
        let l_v1 = common.make_pio_pin(l_v1);
        let l_v0 = common.make_pio_pin(l_v0);
        let h_z0 = common.make_pio_pin(h_z0);
        let nil_0 = common.make_pio_pin(nil_0);
        let nil_1 = common.make_pio_pin(nil_1);
        let nil_2 = common.make_pio_pin(unsafe { led.clone_unchecked() });
        let h_v2 = common.make_pio_pin(h_v2);
        let h_v1 = common.make_pio_pin(h_v1);
        let h_v0 = common.make_pio_pin(h_v0);
        sm.set_pin_dirs(
            Direction::Out,
            &[&l_z0, &l_v2, &l_v1, &l_v0, &h_z0, &h_v2, &h_v1, &h_v0],
        );

        let mut cfg = Config::default();
        cfg.set_out_pins(&[
            &l_z0, &l_v2, &l_v1, &l_v0, &h_z0, &nil_0, &nil_1, &nil_2, &h_v2, &h_v1, &h_v0,
        ]);
        cfg.set_set_pins(&[&l_z0, &l_v2, &l_v1, &l_v0, &h_z0]);
        cfg.use_program(&program.prg, &[]);

        cfg.shift_in.direction = ShiftDirection::Right;
        cfg.shift_out.direction = ShiftDirection::Right;
        cfg.clock_divider = Self::clk_div(baud)?;
        sm.set_config(&cfg);

        Ok(Self {
            sm,
            dma: dma.into(),
            led: Output::new(led, Level::Low),
        })
    }

    pub fn clk_div(baud: u32) -> Result<FixedU32<U8>, TxError> {
        fn _check_clock_div(div: FixedU32<U8>) -> Result<(), TxError> {
            if div < FixedU32::<U8>::from_num(1.0) {
                Err(TxError::ClockDividerTooSmall)
            } else if div > FixedU32::<U8>::from_bits(0xFFFF_FF00) {
                Err(TxError::ClockDividerTooLarge)
            } else {
                Ok(())
            }
        }

        let divisor = 8_u32
            .checked_mul(baud)
            .ok_or(TxError::ClockDividerTooSmall)?;

        let clk_div = (clk_sys_freq() / (divisor)).to_fixed();
        _check_clock_div(clk_div)?;

        Ok(clk_div)
    }

    /// Modify the PIO baud.
    pub fn set_baud(&mut self, baud: u32) -> Result<(), TxError> {
        let clk_div = Self::clk_div(baud)?;

        let enabled = self.is_enabled();

        if enabled {
            self.disable()
        }

        self.sm.set_clock_divider(clk_div);

        if enabled {
            self.enable();
        }

        self.sm.clkdiv_restart();
        self.restart();

        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.sm.is_enabled()
    }

    /// Enable's the PIO program, continuing the wave generation from the PIO program.
    pub fn enable(&mut self) {
        self.sm.set_enable(true);
    }

    /// Stops the PIO program, ceasing all signals from the PIN that were generated via PIO.
    pub fn disable(&mut self) {
        self.sm.set_enable(false);
    }

    pub fn restart(&mut self) {
        self.sm.restart();
    }

    pub async fn write_bytes(&mut self, data: &[u8]) {
        self.led.set_high();
        self.sm
            .tx()
            .dma_push(self.dma.reborrow(), data, false)
            .await;
        self.led.set_low();
    }
}

impl<PIO: Instance, const SM: usize> ErrorType for PioInjector<'_, PIO, SM> {
    type Error = Infallible;
}

impl<PIO: Instance + 'static, const SM: usize> Write for PioInjector<'_, PIO, SM> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Infallible> {
        self.write_bytes(buf).await;
        Ok(buf.len())
    }
}
