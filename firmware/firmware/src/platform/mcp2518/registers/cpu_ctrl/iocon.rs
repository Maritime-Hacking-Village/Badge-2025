use crate::platform::mcp2518::registers::Register;

#[derive(Clone, Copy, defmt::Format)]
pub struct IoCon {
    pub int_od: OpenDrainMode<30>,
    pub sof: SofSignal,
    pub tx_can_od: OpenDrainMode<28>,
    pub pm1: GpioPinMode<25>,
    pub pm0: GpioPinMode<24>,
    pub gpio1: GpioPinStatus<17>,
    pub gpio0: GpioPinStatus<16>,
    pub lat1: GpioPinLatch<9>,
    pub lat0: GpioPinLatch<8>,
    pub x_stby_en: XStbyEn,
    pub tris1: GpioPinDataDirection<1>,
    pub tris0: GpioPinDataDirection<0>,
}

impl Register for IoCon {
    const ADDRESS: u16 = 0xE04;
}

impl From<u32> for IoCon {
    fn from(word: u32) -> Self {
        Self {
            int_od: word.into(),
            sof: word.into(),
            tx_can_od: word.into(),
            pm1: word.into(),
            pm0: word.into(),
            gpio1: word.into(),
            gpio0: word.into(),
            lat1: word.into(),
            lat0: word.into(),
            x_stby_en: word.into(),
            tris1: word.into(),
            tris0: word.into(),
        }
    }
}

impl From<&IoCon> for u32 {
    fn from(reg: &IoCon) -> Self {
        reg.int_od as u32
            | reg.sof as u32
            | reg.tx_can_od as u32
            | reg.pm1 as u32
            | reg.pm0 as u32
            | reg.gpio1 as u32
            | reg.gpio0 as u32
            | reg.lat1 as u32
            | reg.lat0 as u32
            | reg.x_stby_en as u32
            | reg.tris1 as u32
            | reg.tris0 as u32
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum OpenDrainMode<const N: usize> {
    OpenDrainOutput,
    PushPullOutput,
}

impl<const N: usize> From<u32> for OpenDrainMode<N> {
    fn from(word: u32) -> Self {
        match ((word >> N) & 0x01) != 0 {
            true => OpenDrainMode::OpenDrainOutput,
            false => OpenDrainMode::PushPullOutput,
        }
    }
}

impl<const N: usize> From<&OpenDrainMode<N>> for u32 {
    fn from(reg: &OpenDrainMode<N>) -> Self {
        match reg {
            OpenDrainMode::OpenDrainOutput => 1 << N,
            OpenDrainMode::PushPullOutput => 0,
        }
    }
}

impl<const N: usize> From<OpenDrainMode<N>> for u32 {
    fn from(reg: OpenDrainMode<N>) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum GpioPinMode<const N: usize> {
    Gpio,
    Interrupt,
}

impl<const N: usize> From<u32> for GpioPinMode<N> {
    fn from(word: u32) -> Self {
        match ((word >> N) & 0x01) != 0 {
            true => GpioPinMode::Gpio,
            false => GpioPinMode::Interrupt,
        }
    }
}

impl<const N: usize> From<&GpioPinMode<N>> for u32 {
    fn from(reg: &GpioPinMode<N>) -> Self {
        match reg {
            GpioPinMode::Gpio => 1 << N,
            GpioPinMode::Interrupt => 0,
        }
    }
}

impl<const N: usize> From<GpioPinMode<N>> for u32 {
    fn from(reg: GpioPinMode<N>) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum GpioPinStatus<const N: usize> {
    High,
    Low,
}

impl<const N: usize> From<u32> for GpioPinStatus<N> {
    fn from(word: u32) -> Self {
        match ((word >> N) & 0x01) != 0 {
            true => GpioPinStatus::High,
            false => GpioPinStatus::Low,
        }
    }
}

impl<const N: usize> From<&GpioPinStatus<N>> for u32 {
    fn from(reg: &GpioPinStatus<N>) -> Self {
        match reg {
            GpioPinStatus::High => 1 << N,
            GpioPinStatus::Low => 0,
        }
    }
}

impl<const N: usize> From<GpioPinStatus<N>> for u32 {
    fn from(reg: GpioPinStatus<N>) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum GpioPinLatch<const N: usize> {
    High,
    Low,
}

impl<const N: usize> From<u32> for GpioPinLatch<N> {
    fn from(word: u32) -> Self {
        match ((word >> N) & 0x01) != 0 {
            true => GpioPinLatch::High,
            false => GpioPinLatch::Low,
        }
    }
}

impl<const N: usize> From<&GpioPinLatch<N>> for u32 {
    fn from(reg: &GpioPinLatch<N>) -> Self {
        match reg {
            GpioPinLatch::High => 1 << N,
            GpioPinLatch::Low => 0,
        }
    }
}

impl<const N: usize> From<GpioPinLatch<N>> for u32 {
    fn from(reg: GpioPinLatch<N>) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum GpioPinDataDirection<const N: usize> {
    Input,
    Output,
}

impl<const N: usize> From<u32> for GpioPinDataDirection<N> {
    fn from(word: u32) -> Self {
        match ((word >> N) & 0x01) != 0 {
            true => GpioPinDataDirection::Input,
            false => GpioPinDataDirection::Output,
        }
    }
}

impl<const N: usize> From<&GpioPinDataDirection<N>> for u32 {
    fn from(reg: &GpioPinDataDirection<N>) -> Self {
        match reg {
            GpioPinDataDirection::Input => 1 << N,
            GpioPinDataDirection::Output => 0,
        }
    }
}

impl<const N: usize> From<GpioPinDataDirection<N>> for u32 {
    fn from(reg: GpioPinDataDirection<N>) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum XStbyEn {
    Enabled,
    Disabled,
}

impl From<u32> for XStbyEn {
    fn from(word: u32) -> Self {
        match ((word >> 6) & 0x01) != 0 {
            true => XStbyEn::Enabled,
            false => XStbyEn::Disabled,
        }
    }
}

impl From<&XStbyEn> for u32 {
    fn from(reg: &XStbyEn) -> Self {
        match reg {
            XStbyEn::Enabled => 1 << 6,
            XStbyEn::Disabled => 0,
        }
    }
}

impl From<XStbyEn> for u32 {
    fn from(reg: XStbyEn) -> Self {
        u32::from(&reg)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[repr(u8)]
pub enum SofSignal {
    SofOnClkO,
    ClockOnClkO,
}

impl From<u32> for SofSignal {
    fn from(word: u32) -> Self {
        match ((word >> 6) & 0x01) != 0 {
            true => SofSignal::SofOnClkO,
            false => SofSignal::ClockOnClkO,
        }
    }
}

impl From<&SofSignal> for u32 {
    fn from(reg: &SofSignal) -> Self {
        match reg {
            SofSignal::SofOnClkO => 1 << 6,
            SofSignal::ClockOnClkO => 0,
        }
    }
}

impl From<SofSignal> for u32 {
    fn from(reg: SofSignal) -> Self {
        u32::from(&reg)
    }
}
