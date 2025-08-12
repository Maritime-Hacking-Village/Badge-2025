//! Derived from the `nmea0183` crate.

use crate::apps::rx::SerialParser;
use alloc::string::{String, ToString};
use defmt::{debug, Format};

pub const NMEA0183_MTU: usize = 120;

fn parse_hex_nibble(symbol: u8) -> Option<u8> {
    if symbol >= b'0' && symbol <= b'9' {
        Some(symbol - b'0')
    } else if symbol >= b'A' && symbol <= b'F' {
        Some(symbol - b'A' + 10)
    } else {
        None
    }
}

fn from_ascii(bytes: &[u8]) -> Option<&str> {
    if bytes.iter().all(|b| *b < 128) {
        Some(unsafe { core::str::from_utf8_unchecked(bytes) })
    } else {
        None
    }
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum State {
    WaitStart,
    ReadUntilChkSum,
    ChkSumUpper,
    ChkSumLower,
    WaitCR,
    WaitLF,
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    TooLong,
    Checksum(u8, u8),
    InvalidHex,
    InvalidAscii,
    Format,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl core::error::Error for Error {}

pub struct Parser {
    buffer: [u8; NMEA0183_MTU],
    buflen: usize,
    sof: u8,
    chksum: u8,
    expected_chksum: u8,
    state: State,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            buffer: [0u8; NMEA0183_MTU],
            buflen: 0,
            sof: 0,
            chksum: 0,
            expected_chksum: 0,
            state: State::WaitStart,
        }
    }
}

impl SerialParser for Parser {
    type Word = u8;
    type Message = (u8, String, u8);
    type Error = Error;

    fn parse_word(&mut self, word: Self::Word) -> Option<Result<Self::Message, Self::Error>> {
        debug!("STATE {:?}: {}", self.state, word);

        let (new_state, result) = match self.state {
            State::WaitStart if word == b'$' || word == b'!' => {
                self.sof = word;
                self.buflen = 0;
                self.chksum = 0;
                (State::ReadUntilChkSum, None)
            }
            State::WaitStart if word != b'$' && word != b'!' => (State::WaitStart, None),
            State::ReadUntilChkSum if word != b'*' => {
                if self.buffer.len() <= self.buflen {
                    (State::WaitStart, Some(Err(Error::TooLong)))
                } else {
                    self.buffer[self.buflen] = word;
                    self.buflen += 1;
                    self.chksum = self.chksum ^ word;
                    (State::ReadUntilChkSum, None)
                }
            }
            State::ReadUntilChkSum if word == b'*' => (State::ChkSumUpper, None),
            State::ChkSumUpper => match parse_hex_nibble(word) {
                Some(s) => {
                    self.expected_chksum = s;
                    (State::ChkSumLower, None)
                }
                None => (State::WaitStart, Some(Err(Error::InvalidHex))),
            },
            State::ChkSumLower => match parse_hex_nibble(word) {
                Some(s) => {
                    if ((self.expected_chksum << 4) | s) != self.chksum {
                        (
                            State::WaitStart,
                            Some(Err(Error::Checksum(self.expected_chksum, self.chksum))),
                        )
                    } else {
                        (State::WaitCR, None)
                    }
                }
                None => (State::WaitStart, Some(Err(Error::InvalidHex))),
            },
            State::WaitCR if word == b'\r' => (State::WaitLF, None),
            State::WaitLF if word == b'\n' => match from_ascii(&self.buffer[..self.buflen]) {
                Some(s) => (
                    State::WaitStart,
                    Some(Ok((self.sof, s.to_string(), self.chksum))),
                ),
                None => (State::WaitStart, Some(Err(Error::InvalidAscii))),
            },
            _ => (State::WaitStart, Some(Err(Error::Format))),
        };

        self.state = new_state;
        result
    }

    fn reset(&mut self) {
        self.buffer.fill(0x00);
        self.buflen = 0;
        self.sof = 0;
        self.chksum = 0;
        self.expected_chksum = 0;
        self.state = State::WaitStart;
    }

    fn mtu() -> usize {
        NMEA0183_MTU
    }

    fn default_baud() -> u32 {
        38_400
    }
}
