#[derive(Copy, Clone, Debug)]
pub enum TokenError {
    NotToken,
    Generic,
    CC,
    CardECC,
    OutOfRange,
    CardLocked,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Token {
    Start = 0xFE,
    StartWriteMultipleBlock = 0xFC,
    Stop = 0xFD,
}

impl TryFrom<u8> for Token {
    type Error = TokenError;

    fn try_from(byte: u8) -> Result<Token, TokenError> {
        match (byte, byte ^ (byte & byte.wrapping_sub(1))) {
            (0xFE, _) => Ok(Token::Start),
            (0xFC, _) => Ok(Token::StartWriteMultipleBlock),
            (_, 0x10) => Err(TokenError::CardLocked),
            (_, 0x8) => Err(TokenError::OutOfRange),
            (_, 0x4) => Err(TokenError::CardECC),
            (_, 0x2) => Err(TokenError::CC),
            (_, 0x1) => Err(TokenError::Generic),
            (_, _) => Err(TokenError::NotToken),
        }
    }
}

pub enum Response {
    Accepted,
    CRCError,
    WriteError,
}

impl Response {
    pub fn try_from(byte: u8) -> Option<Self> {
        if byte & 0b10001 != 0x1 {
            return None;
        }
        let value = match (byte >> 1) & 0b111 {
            0b010 => Self::Accepted,
            0b101 => Self::CRCError,
            0b110 => Self::WriteError,
            _ => return None,
        };
        Some(value)
    }
}
