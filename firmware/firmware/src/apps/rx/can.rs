use crate::apps::rx::SerialParser;
use core::convert::Infallible;
use defmt::{debug, error, warn, Format};
use embassy_futures::{select, select::Either3};
use embassy_rp::{
    clocks::clk_sys_freq,
    pio::{
        Common, Config, Direction as PioDirection, Instance, Irq, LoadedProgram, PioPin,
        ShiftDirection, StateMachine,
    },
    Peri,
};
use embedded_io_async::ErrorType;
use fixed::traits::ToFixed;

// MTU as bytes
pub const CAN_2B_MTU: usize = 22;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum CanWord {
    Sof,
    Word(u32),
    Ifs,
}

/// This struct represents a Uart Rx program loaded into pio instruction memory.
pub struct PioCanRxProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
}

impl<'d, PIO: Instance> PioCanRxProgram<'d, PIO> {
    /// Load the uart rx program into the given pio
    pub fn new(common: &mut Common<'d, PIO>) -> Self {
        let prg = pio::pio_asm!(
            r#"
                wait_ifs:
                    set x, 0b11                 ; we need 3 recessive bits before we start
                    mov isr, x                  ; 3 bits * 16 (oversample) / 2 (instructions) = 24 = 0x18 = 0b11<<3
                    in null, 3                  ; shift left 3
                ifs_loop:
                    jmp x-- idle                ; got enough ifs, jump to idle
                    jmp pin ifs_loop            ; recessive just keep counting
                    jmp wait_ifs                ; got dominant, reset counter
                idle:
                    mov isr, null               ; clear ISR
                    wait 0 pin 0                ; wait for dominant bit
                    irq 0 [3]                   ; IRQ0: SOF
                receive_bit:
                    in pins, 1                  ; shift bit into ISR
                    jmp pin recessive [4]       ; got recessive bit
                    set x, 6                    ; dominant, reset counter
                    jmp receive_bit
                recessive:
                    jmp x-- receive_bit [1]     ; did not get EOF
                    irq 1                       ; Got EOF (irq1)
                    push                        ; flush pending data
                    jmp wait_ifs
            "#
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// PIO backed Uart reciever
pub struct PioCanRx<'d, PIO: Instance, const SM: usize> {
    sm_rx: StateMachine<'d, PIO, SM>,
    irq_sof: Irq<'d, PIO, 0>,
    irq_ifs: Irq<'d, PIO, 1>,
}

impl<'d, PIO: Instance, const SM: usize> PioCanRx<'d, PIO, SM> {
    /// Configure a pio state machine to use the loaded rx program.
    pub fn new(
        baud: u32,
        common: &mut Common<'d, PIO>,
        mut sm_rx: StateMachine<'d, PIO, SM>,
        rx_pin: Peri<'d, impl PioPin>,
        // debug_pin: Peri<'d, impl PioPin>,
        program: &PioCanRxProgram<'d, PIO>,
        irq_sof: Irq<'d, PIO, 0>,
        irq_ifs: Irq<'d, PIO, 1>,
    ) -> Self {
        let mut cfg = Config::default();
        // let debug_pin = common.make_pio_pin(debug_pin);
        cfg.use_program(&program.prg, &[]);

        let rx_pin = common.make_pio_pin(rx_pin);
        // sm_rx.set_pins(Level::High, &[&rx_pin]);
        // cfg.set_set_pins(&[&debug_pin]);
        cfg.set_in_pins(&[&rx_pin]);
        cfg.set_jmp_pin(&rx_pin);
        sm_rx.set_pin_dirs(PioDirection::In, &[&rx_pin]);
        // sm_rx.set_pin_dirs(PioDirection::Out, &[&debug_pin]);

        // TODO: Check if the clock divider is an integer.
        cfg.clock_divider = (clk_sys_freq() / (8 * baud)).to_fixed();
        warn!(
            "SYS CLOCK: {:?} CLOCK DIV: {:?}",
            defmt::Debug2Format(&clk_sys_freq()),
            defmt::Debug2Format(&cfg.clock_divider)
        );
        cfg.shift_in.auto_fill = true;
        cfg.shift_in.direction = ShiftDirection::Left;
        cfg.shift_in.threshold = 32;
        // cfg.fifo_join = FifoJoin::RxOnly;
        sm_rx.set_config(&cfg);

        // flush
        let rx = sm_rx.rx();
        while let Some(_) = rx.try_pull() {}
        sm_rx.restart();

        Self {
            sm_rx,
            irq_sof,
            irq_ifs,
        }
    }

    pub fn enable(&mut self) {
        if !self.sm_rx.is_enabled() {
            self.sm_rx.set_enable(true);
        }
    }

    pub fn disable(&mut self) {
        if self.sm_rx.is_enabled() {
            self.sm_rx.set_enable(false);
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.sm_rx.is_enabled()
    }

    pub async fn read_word(&mut self) -> CanWord {
        match select::select3(
            self.irq_sof.wait(),
            self.sm_rx.rx().wait_pull(),
            self.irq_ifs.wait(),
        )
        .await
        {
            Either3::First(_) => CanWord::Sof,
            Either3::Second(word) => CanWord::Word(word),
            Either3::Third(_) => CanWord::Ifs,
        }
    }
}

impl<PIO: Instance, const SM: usize> ErrorType for PioCanRx<'_, PIO, SM> {
    type Error = Infallible;
}

// TODO: Maybe don't want this since we read byte-by-byte.
// impl<PIO: Instance, const SM: usize> Read for PioCanRx<'_, PIO, SM> {
//     async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Infallible> {
//         let mut i = 0;

//         while i < buf.len() {
//             buf[i] = self.read_byte().await;
//             i += 1;
//         }

//         Ok(i)
//     }
// }

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Sof,
    Start0,
    Start1(u32),
    Start2([u32; 2]),
    Start3([u32; 3]),
    Start4([u32; 4]),
    Start5([u32; 5]),
    Eof([u32; 6], usize),
    // Sof,
    // IdA,
    // // ID_A, RTR, DLC, DATA0, bits remaining on b3, number of stuff bits, SB state
    // Data(u16, bool, u8, u8, u8, usize, (bool, usize)),
    // // ID_A, SRR, ID_B, RTR, DLC, bit remaining on b3, SB state
    // DataExt(u16, bool, u32, bool, u8, bool, (bool, usize)),
    // // ID_A, SRR, ID_B, RTR, DLC, SB state
    // DataExt1Sb(u16, bool, u32, bool, u8, (bool, usize)),
    // // ID_A, SRR, ID_B, RTR, DLC, number of stuff bits, SB state
    // DataExtNSb(u16, bool, u32, bool, u8, usize, (bool, usize)),
    // Crc,
    // CrcDelim,
    // Ack,
    // AckDelim,
    // Eof,
    // Ifs,
    // // Extended identifier frames.
    // Srr,
    // R1,
    // IdB,
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub struct Message {
    /// Arbitration ID
    arb_id: u32,
    /// Remote transmit request
    rtr: bool,
    /// Number of payload bytes present
    dlc: u8,
    /// Message payload
    payload: [u8; 8],
    /// CRC-16
    crc: u16,
    /// Acknowledgement
    ack: bool,
}

impl Message {
    pub fn is_extended(&self) -> bool {
        (self.arb_id >> 12) > 0
    }
}

// TODO
#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidSof,
    ExpectedSrr,
    InvalidR1,
    InvalidR0,
    InvalidDlc(u8),
    InvalidCrcDelim,
    InvalidAckDelim,
    InvalidChecksum(u16, u16),
    InvalidEof,
    InvalidIfs,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl core::error::Error for Error {}

pub struct Parser {
    state: State,
}

impl Parser {
    pub fn new() -> Self {
        Parser { state: State::Sof }
    }
}

fn contains_eof(word: u32) -> bool {
    let mut cnt = 0;

    for i in 0..32 {
        let bit = ((word >> i) & 0b1) != 0;

        if bit {
            cnt += 1;
        } else {
            break;
        }
    }

    debug!("EOF COUNT {}", cnt);
    if cnt == 7 {
        true
    } else {
        false
    }
}

fn remove_stuff_bits(
    word: u32,
    maybe_prev_state: Option<(bool, usize)>,
) -> (u32, usize, (bool, usize)) {
    let mut stuff_bits: usize = 0;
    let mut new_word: u32 = (word & 0x80000000) >> 31;
    let mut cursor: bool = new_word != 0;
    let mut cnt = 1;
    let mut skip = false;

    if let Some((prev_cursor, prev_run_length)) = maybe_prev_state {
        debug!("PREV CURSOR {} {}", prev_cursor, prev_run_length);

        // Update our cursor to include the previous word.
        if cursor == prev_cursor {
            cnt = prev_run_length + 1;

            if cnt >= 6 {
                error!("Something went wrong w/ the stuff bit removal: {}", cnt);
            }

            if cnt >= 5 {
                cnt = 0;
                stuff_bits += 1;
                skip = true;
            }
        }
    }

    for i in 1..32 {
        let bit = ((word << i) & 0x80000000) != 0;

        if skip {
            skip = false;
            debug!(
                "Skipping index {}: {} != {} {:08X}",
                i, bit, cursor, new_word
            );
            // assert!(bit != cursor);
            continue;
        }

        if bit == cursor {
            cnt += 1;
        } else {
            cursor = bit;
            cnt = 1;
        }

        new_word <<= 1;
        new_word |= bit as u32;

        if cnt == 5 {
            cnt = 0;
            stuff_bits += 1;
            skip = true;
        }
    }

    // Count the run length of the end of the word for the next pass.
    let mut run_length = 1;
    let cursor = (new_word & 0b1) != 0;

    for i in 1..(32 - stuff_bits) {
        let bit = ((new_word >> i) & 0b1) != 0;

        if bit == cursor {
            run_length += 1;
        } else {
            break;
        }
    }

    new_word <<= stuff_bits;

    (new_word, stuff_bits, (cursor, run_length))
}

impl SerialParser for Parser {
    type Word = CanWord;
    type Message = ();
    type Error = Error;

    fn parse_word(&mut self, word: Self::Word) -> Option<Result<Self::Message, Self::Error>> {
        warn!("CAN Parser: [{:?}] {:?}", self.state, word);

        match word {
            CanWord::Sof => {
                if self.state != State::Sof {
                    self.state = State::Sof;
                    return Some(Err(Error::InvalidSof));
                }

                self.state = State::Start0;
            }
            CanWord::Word(word) => {
                let has_eof = contains_eof(word);
                warn!("GOT WORD {:08X} EOF {}", word, has_eof);

                match self.state {
                    State::Sof => {
                        return Some(Err(Error::InvalidSof));
                    }
                    State::Start0 => {
                        if has_eof {
                            error!("Start0 contains EOF!");
                            self.state = State::Sof;
                            return Some(Err(Error::InvalidEof));
                        }

                        self.state = State::Start1(word);
                    }
                    State::Start1(w0) => {
                        if has_eof {
                            warn!("Start1 contains EOF!");
                            self.state = State::Eof([w0, word, 0, 0, 0, 0], 2);
                        } else {
                            self.state = State::Start2([w0, word]);
                        }
                    }
                    State::Start2(ws) => {
                        if has_eof {
                            warn!("Start2 contains EOF!");
                            self.state = State::Eof([ws[0], ws[1], word, 0, 0, 0], 3);
                        } else {
                            self.state = State::Start3([ws[0], ws[1], word]);
                        }
                    }
                    State::Start3(ws) => {
                        if has_eof {
                            warn!("Start3 contains EOF!");
                            self.state = State::Eof([ws[0], ws[1], ws[2], word, 0, 0], 4);
                        } else {
                            self.state = State::Start4([ws[0], ws[1], ws[2], word]);
                        }
                    }
                    State::Start4(ws) => {
                        if has_eof {
                            warn!("Start4 contains EOF!");
                            self.state = State::Eof([ws[0], ws[1], ws[2], ws[3], word, 0], 5);
                        } else {
                            self.state = State::Start5([ws[0], ws[1], ws[2], ws[3], word]);
                        }
                    }
                    State::Start5(ws) => {
                        if has_eof {
                            warn!("Start5 contains EOF!");
                            self.state = State::Eof([ws[0], ws[1], ws[2], ws[3], ws[4], word], 6);
                        } else {
                            self.state = State::Sof;
                            return Some(Err(Error::InvalidEof));
                        }
                    }
                    State::Eof(_, _) => {
                        self.state = State::Sof;
                        return Some(Err(Error::InvalidIfs));
                    }
                }
            }
            CanWord::Ifs => {
                error!("EOF!!!: {:?}", self.state);

                match self.state {
                    State::Eof(ws, len) => {
                        self.state = State::Sof;
                        assert!(len > 0);
                        let mut ws_unstuffed: [u32; 6] = [0; 6];
                        let mut sbs: [usize; 6] = [0; 6];
                        let mut sb_state: Option<(bool, usize)> = None;

                        for i in 0..len {
                            warn!("EOF {}: {:08X}", i, ws[i]);
                            let (unstuffed, sb, sb_state_new) = remove_stuff_bits(ws[i], sb_state);
                            sb_state = Some(sb_state_new);
                            ws_unstuffed[i] = unstuffed;
                            sbs[i] = sb;
                        }

                        // // Blow away the EOF.
                        // ws[len - 1] >>= 7;
                        // ws[len - 1] <<= 7;
                    }
                    _ => {
                        self.state = State::Sof;
                        return Some(Err(Error::InvalidIfs));
                    }
                }
            }
        }

        None
    }

    // I realized I can do this much easier.
    // fn parse_word(&mut self, word: Self::Word) -> Option<Result<Self::Message, Self::Error>> {
    //     warn!("CAN Parser: [{:?}] {:?}", self.state, word);

    //     match word {
    //         CanWord::Sof => {
    //             if self.state != State::Sof {
    //                 return Some(Err(Error::InvalidSof));
    //             }

    //             self.state = State::IdA;
    //         }
    //         CanWord::Word(word) => {
    //             match self.state {
    //                 State::IdA => {
    //                     debug!("OLD WORD: {:08X}", word);
    //                     let (word, stuff_bits, sb_state) = remove_stuff_bits(word, None);
    //                     debug!("NEW WORD: {:08X} STUFF BITS: {}", word, stuff_bits);
    //                     let b3 = (word & 0x000000FF) as u8;
    //                     let b2 = ((word & 0x0000FF00) >> 8) as u8;
    //                     let b1 = ((word & 0x00FF0000) >> 16) as u8;
    //                     let b0 = ((word & 0xFF000000) >> 24) as u8;
    //                     debug!("{:02X} {:02X} {:02X} {:02X}", b0, b1, b2, b3);

    //                     let id_a: u16 = ((b0 as u16) << 4) | (((b1 & 0b11110000) >> 4) as u16);
    //                     let srr_or_rtr: bool = b1 & 0b00001000 != 0;
    //                     let ide: bool = b1 & 0b00000100 != 0;

    //                     debug!("ID A: {:04X}", id_a);
    //                     debug!("SRR/RTR: {}", srr_or_rtr);
    //                     debug!("IDE: {}", ide);

    //                     if ide {
    //                         // Extended frame
    //                         debug!("Extended frame!");
    //                         let srr = srr_or_rtr;

    //                         if !srr {
    //                             self.state = State::Sof;
    //                             return Some(Err(Error::ExpectedSrr));
    //                         }

    //                         let id_b: u32 = (((b1 & 0b11) as u32) << 16) | (b2 as u32);
    //                         let rtr: bool = b3 & 0x80 != 0;
    //                         let r1: bool = b3 & 0x40 != 0;

    //                         debug!("ID B: {:08X}", id_b);
    //                         debug!("RTR: {}", rtr);
    //                         debug!("R1: {}", r1);

    //                         if r1 {
    //                             self.state = State::Sof;
    //                             return Some(Err(Error::InvalidR1));
    //                         }

    //                         let r0 = b3 & 0x20 != 0;

    //                         debug!("R0: {}", r0);

    //                         if r0 {
    //                             self.state = State::Sof;
    //                             return Some(Err(Error::InvalidR0));
    //                         }

    //                         // NOTE: Have to start caring about stuff bits now.
    //                         //       We can have at most 5 stuff bits in a word.
    //                         match stuff_bits {
    //                             0 => {
    //                                 let dlc: u8 = (b3 & 0b11110) >> 3;
    //                                 debug!("DLC: {}", dlc);
    //                                 self.state = State::DataExt(
    //                                     id_a,
    //                                     srr,
    //                                     id_b,
    //                                     rtr,
    //                                     dlc,
    //                                     (b3 & 0b1) != 0,
    //                                     sb_state,
    //                                 );
    //                                 return None;
    //                             }
    //                             1 => {
    //                                 let dlc: u8 = (b3 & 0b11110) >> 3;
    //                                 debug!("DLC: {}", dlc);
    //                                 self.state =
    //                                     State::DataExt1Sb(id_a, srr, id_b, rtr, dlc, sb_state);
    //                                 return None;
    //                             }
    //                             _ => {
    //                                 // NOTE: This will not fill the DLC field since the number of stuff bits
    //                                 //       bleeds into the DLC's length. We have to manage this next word.
    //                                 let dlc: u8 = (b3 & 0b11110) >> 3;
    //                                 debug!("DLC: {}", dlc);
    //                                 self.state = State::DataExtNSb(
    //                                     id_a, srr, id_b, rtr, dlc, stuff_bits, sb_state,
    //                                 );
    //                                 return None;
    //                             }
    //                         }
    //                     } else {
    //                         let rtr = srr_or_rtr;
    //                         let r0 = b1 & 0b10 != 0;

    //                         debug!("Standard frame!");
    //                         debug!("R0: {}", r0);

    //                         if r0 {
    //                             self.state = State::Sof;
    //                             return Some(Err(Error::InvalidR0));
    //                         }

    //                         let dlc: u8 = ((b1 & 0b1) << 3) | ((b2 & 0xE0) >> 5);

    //                         debug!("DLC: {}", dlc);

    //                         if dlc > 8 {
    //                             self.state = State::Sof;
    //                             return Some(Err(Error::InvalidDlc(dlc)));
    //                         }

    //                         let data_0: u8 = ((b2 & 0b11111) << 3) | ((b3 & 0xE0) >> 3);

    //                         // NOTE: Have to start caring about stuff bits now.
    //                         //       We can have at most 5 stuff bits in a word.
    //                         self.state = State::Data(
    //                             id_a,
    //                             rtr,
    //                             dlc,
    //                             data_0,
    //                             b3 & 0b11111,
    //                             stuff_bits,
    //                             sb_state,
    //                         );
    //                         return None;
    //                     }
    //                 }
    //                 State::Data(id_a, rtr, dlc, data_0, old_b3, old_stuff_bits, old_sb_state) => {
    //                     warn!(
    //                         "[DATA] ID_A {:04X} RTR: {} DLC: {} DATA0: {:02X} B3: {:02X} STUFF_BITS {} SB_STATE: {:?}",
    //                         id_a, rtr, dlc, data_0, old_b3, old_stuff_bits, old_sb_state
    //                     );
    //                     debug!("OLD WORD: {:08X}", word);
    //                     let has_eof = contains_eof(word);
    //                     warn!("HAS_EOF: {}", has_eof);
    //                     let (word, stuff_bits, sb_state) =
    //                         remove_stuff_bits(word, Some(old_sb_state));
    //                     debug!("NEW WORD: {:08X} STUFF BITS: {}", word, stuff_bits);
    //                     let b3 = (word & 0x000000FF) as u8;
    //                     let b2 = ((word & 0x0000FF00) >> 8) as u8;
    //                     let b1 = ((word & 0x00FF0000) >> 16) as u8;
    //                     let b0 = ((word & 0xFF000000) >> 24) as u8;
    //                     debug!("{:02X} {:02X} {:02X} {:02X}", b0, b1, b2, b3);

    //                     let mut data: [u8; 8] = [0; 8];
    //                     let mut crc: u16 = 0x0000;

    //                     match dlc {
    //                         0 => {
    //                             // Remove bits from the `data_0` for next element.
    //                             crc |= (data_0 as u16) << 8;
    //                             // Remove any valid bits from `old_b3`.
    //                             crc |= (old_b3 << 3) as u16;
    //                             debug!("Shifting left {}", 3 + old_stuff_bits);
    //                             crc |= (b0 as u16) << (3 + old_stuff_bits);
    //                             // Now we have the first byte of the CRC loaded.
    //                             // 7 more bits to go.
    //                             crc |= (b1 as u16) << 1;
    //                             // We have 1 bit left on b1.
    //                             // Need to shift the CRC right by one bit since it's 15-bit.
    //                             crc >>= 1;
    //                             debug!("CRC: {:02X}", crc);
    //                             let crc_delim: bool = ((b1 & 0b1) << 1) != 0;
    //                             debug!("CRC_DELIM: {}", crc_delim);

    //                             if crc_delim {
    //                                 self.state = State::Sof;
    //                                 return Some(Err(Error::InvalidCrcDelim));
    //                             }

    //                             let ack = (b2 >> 7) != 0;
    //                             debug!("ACK: {}", ack);
    //                             let ack_delim = (b2 >> 6) != 0;
    //                             debug!("ACK_DELIM: {}", ack_delim);

    //                             if !ack_delim {
    //                                 self.state = State::Sof;
    //                                 return Some(Err(Error::InvalidAckDelim));
    //                             }
    //                         }
    //                         1 => {
    //                             // TODO
    //                             // Remove any valid bits from `old_b3`.
    //                             let test: u128 = 0x00;
    //                             crc |= ((old_b3 << 3) as u16) << 8;
    //                         }
    //                         _ => {
    //                             // TODO
    //                             // Remove any valid bits from `old_b3`.
    //                         }
    //                     }

    //                     self.state = State::Sof;
    //                     return Some(Err(Error::InvalidState));
    //                 }
    //                 _ => {
    //                     self.state = State::Sof;
    //                     return Some(Err(Error::InvalidState));
    //                 }
    //             }
    //         }
    //         CanWord::Eof => {
    //             if self.state != State::Eof {
    //                 return Some(Err(Error::InvalidEof));
    //             }

    //             self.state = State::Sof;
    //         }
    //     };

    //     None
    // }

    fn reset(&mut self) {
        self.state = State::Sof;
    }

    fn mtu() -> usize {
        CAN_2B_MTU
    }

    fn default_baud() -> u32 {
        250_000
    }
}
