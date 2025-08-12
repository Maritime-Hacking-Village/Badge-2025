use defmt::Format;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Value<const B: usize, const O: u32, const S: u32, const D: u8>(u8);

impl<const B: usize, const O: u32, const S: u32, const D: u8> Value<B, O, S, D> {
    pub fn new(value: u32) -> Self {
        let byte = (value - O) as u32 / S;
        assert!(byte < 1 << B);
        Self(byte as u8)
    }

    pub fn get_value(&self) -> u32 {
        self.0 as u32 * S + O
    }
}

impl<const B: usize, const O: u32, const S: u32, const D: u8> Default for Value<B, O, S, D> {
    fn default() -> Self {
        Self(D)
    }
}

impl<const B: usize, const O: u32, const S: u32, const D: u8> From<u8> for Value<B, O, S, D> {
    fn from(b: u8) -> Self {
        assert!(b < 1 << B);
        Self(b)
    }
}

impl<const B: usize, const O: u32, const S: u32, const D: u8> From<Value<B, O, S, D>> for u8 {
    fn from(value: Value<B, O, S, D>) -> Self {
        value.0
    }
}

impl<const B: usize, const O: u32, const S: u32, const D: u8> From<u32> for Value<B, O, S, D> {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl<const B: usize, const O: u32, const S: u32, const D: u8> From<&Value<B, O, S, D>> for u8 {
    fn from(value: &Value<B, O, S, D>) -> Self {
        value.0
    }
}

impl<const B: usize, const O: u32, const S: u32, const D: u8> core::fmt::Display
    for Value<B, O, S, D>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Value {{ {:#08b}: {} }}", self.0, self.get_value())
    }
}

impl<const B: usize, const O: u32, const S: u32, const D: u8> Format for Value<B, O, S, D> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Value {{ {:#08b}: {} }}", self.0, self.get_value())
    }
}
