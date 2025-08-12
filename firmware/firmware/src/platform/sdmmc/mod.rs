pub mod card_type;
pub mod command;
pub mod registers;
pub mod response;
pub mod spi;
pub mod transfer;

use response::R1Status;

pub const BLOCK_SIZE: usize = 512;

#[derive(Debug)]
pub enum Error<E> {
    /// Bus error
    BUS(E),
    /// Probably no card
    NoResponse,
    /// Not idle
    NotIdle,
    /// Command related error
    Command(R1Status),
    /// Transfer error
    Transfer(transfer::TokenError),
    /// No respond within expected duration
    Timeout,
    /// Unexpected error
    Generic,
    /// Chip Select Error
    ChipSelect,
}

// impl<T: SpiBus + ErrorType> From<T::Error> for Error<T::Error> {
//     fn from(value: T::Error) -> Self {
//         Error::BUS(value)
//     }
// }

// impl<T: SpiBus> From<T::Error> for Error<T::Error> {
//     fn from(error: T::Error) -> Self {
//         Self::BUS(error)
//     }
// }

// impl<ERR> From<ERR> for Error<ERR> {
//     fn from(error: ERR) -> Self {
//         Self::BUS(error)
//     }
// }

// impl<BUS: Bus> embedded_hal::digital::ErrorType for BUS {
//     type Error = Error<BUS>;
// }
