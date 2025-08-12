#[derive(Copy, Clone, Debug)]
pub enum CardType {
    SDSC(u8),
    SDHC,
}

impl CardType {
    pub fn high_capacity(self) -> bool {
        match self {
            Self::SDSC(_) => false,
            _ => true,
        }
    }
}
