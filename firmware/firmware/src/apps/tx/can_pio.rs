//! Pio backed uart drivers

use embassy_rp::{
    clocks::clk_sys_freq,
    pio::{
        Common, Config, Direction, Instance, LoadedProgram, PioPin, ShiftDirection, StateMachine,
    },
    Peri,
};
use fixed::{traits::ToFixed, types::extra::U8, FixedU32};

use crate::apps::tx::TxError;

/// This struct represents a uart tx program loaded into pio instruction memory.
pub struct PioCanTrxProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
}

impl<'d, PIO: Instance> PioCanTrxProgram<'d, PIO> {
    /// Load the uart tx program into the given pio
    pub fn new(common: &mut Common<'d, PIO>) -> Self {
        // let prg = pio::pio_asm!(
        //     r#"
        //     .define DOMINANT 0b010
        //     .define RECESSIVE 0b101

        //     wait_ifs:
        //         set pins, RECESSIVE         ; OUTPUT recessive
        //     ifs_loop_reset:
        //         set x, 0b11                 ; we need 3 recessive bits before we start
        //         mov isr, x                  ; 7 bits * 16 (oversample) / 2 (instructions) = 24 = 0x18
        //         in null, 3                  ; shift left 3
        //     ifs_loop:
        //         jmp x-- idle                ; got ifs, jump to idle
        //         jmp pin ifs_loop            ; recessive just keep counting
        //         jmp ifs_loop_reset          ; got dominant, reset counter
        //     idle:
        //         jmp !osre tx_bit
        //     eof:
        //         irq 0 [1]                   ; IRQ0 = EOF
        //         set pins, RECESSIVE         ; OUTPUT recessive
        //         set x, 7                    ; eof is recessive for 7 bits
        //     eof_loop:
        //         jmp x-- eof_loop [15]
        //         jmp wait_ifs
        //     tx_bit:
        //         out x, 1                    ; get next bit
        //         jmp !x dominant
        //     recessive:
        //         set pins, RECESSIVE [7]     ; OUTPUT recessive
        //         jmp pin idle [3]            ; check for recessive
        //         irq 1                       ; IRQ1 = arbitration error
        //         jmp wait_ifs                ; reset
        //     dominant:
        //         set pins, DOMINANT [7]      ; OUTPUT dominant
        //         jmp idle [3]
        // "#
        // );
        let prg = pio::pio_asm!(
            r#"
            .side_set 3 opt
            .define DOMINANT         0b00100
            .define RECESSIVE        0b10011
            .define STRONG_RECESSIVE 0b00010
            .define SIDE_RECESSIVE   0b001
            .define SIDE_DOMINANT    0b011

            ; initialize x to be 0xffffffff
            mov x, ! null

            loop:
                jmp !osre prepare
                pull noblock
            inner:
                out y, 1
                jmp !y dominant
            recessive:
                set pins RECESSIVE side SIDE_RECESSIVE
                jmp loop
            dominant:
                set pins DOMINANT side SIDE_DOMINANT
                jmp loop
            prepare:
                jmp inner
        "#
        );
        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// PIO backed Uart transmitter
pub struct PioCanTrx<'d, PIO: Instance, const SM: usize> {
    sm: StateMachine<'d, PIO, SM>,
}

impl<'d, PIO: Instance, const SM: usize> PioCanTrx<'d, PIO, SM> {
    /// Configure a pio state machine to use the loaded tx program.
    pub fn new(
        baud: u32,
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        l_z0: Peri<'d, impl PioPin>,
        l_v2: Peri<'d, impl PioPin>,
        l_v1: Peri<'d, impl PioPin>,
        l_v0: Peri<'d, impl PioPin>,
        h_z0: Peri<'d, impl PioPin>,
        h_v2: Peri<'d, impl PioPin>,
        h_v1: Peri<'d, impl PioPin>,
        h_v0: Peri<'d, impl PioPin>,
        program: &PioCanTrxProgram<'d, PIO>,
    ) -> Result<Self, TxError> {
        let l_z0 = common.make_pio_pin(l_z0);
        let l_v2 = common.make_pio_pin(l_v2);
        let l_v1 = common.make_pio_pin(l_v1);
        let l_v0 = common.make_pio_pin(l_v0);
        let h_z0 = common.make_pio_pin(h_z0);
        let h_v2 = common.make_pio_pin(h_v2);
        let h_v1 = common.make_pio_pin(h_v1);
        let h_v0 = common.make_pio_pin(h_v0);
        sm.set_pin_dirs(
            Direction::Out,
            &[&l_z0, &l_v2, &l_v1, &l_v0, &h_z0, &h_v2, &h_v1, &h_v0],
        );

        let mut cfg = Config::default();
        cfg.set_set_pins(&[&l_z0, &l_v2, &l_v1, &l_v0, &h_z0]);
        cfg.use_program(&program.prg, &[&h_v2, &h_v1, &h_v0]);
        cfg.shift_out.direction = ShiftDirection::Left;
        cfg.clock_divider = Self::clk_div(baud)?;
        sm.set_config(&cfg);

        Ok(Self { sm })
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

        let divisor = 6_u32
            .checked_mul(baud)
            .ok_or(TxError::ClockDividerTooSmall)?;

        let clk_div = (clk_sys_freq() / divisor).to_fixed();
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
            self.enable()
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

    /// Write a single u8
    pub async fn write_word(&mut self, word: u32) {
        self.sm.tx().wait_push(word).await;
    }
}

// impl<PIO: Instance, const SM: usize> ErrorType for PioCanTrx<'_, PIO, SM> {
//     type Error = Infallible;
// }

// impl<PIO: Instance, const SM: usize> Write for PioCanTrx<'_, PIO, SM> {
//     async fn write(&mut self, buf: &[u8]) -> Result<usize, Infallible> {
//         for byte in buf {
//             self.write_u8(*byte).await;
//         }
//         Ok(buf.len())
//     }
// }
