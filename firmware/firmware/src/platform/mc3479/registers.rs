use defmt::Format;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum Registers {
    DevStat = 0x05,
    IntrCtrl = 0x06,
    Mode = 0x07,
    SampleRate = 0x08,
    MotionCtrl = 0x09,
    FifoStat = 0x0A,
    FifoRdPtr = 0x0B,
    FifoWrPtr = 0x0C,
    XOutExL = 0x0D,
    XOutExH = 0x0E,
    YOutExL = 0x0F,
    YOutExH = 0x10,
    ZOutExL = 0x11,
    ZOutExH = 0x12,
    Status = 0x13,
    IntrStat = 0x14,
    ChipId = 0x18,
    Range = 0x20,
    XOffL = 0x21,
    XOffH = 0x22,
    YOffL = 0x23,
    YOffH = 0x24,
    ZOffL = 0x25,
    ZOffH = 0x26,
    XGain = 0x27,
    YGain = 0x28,
    ZGain = 0x29,
    FifoCtrl = 0x2D,
    FifoTh = 0x2E,
    FifoIntr = 0x2F,
    FifoCtrl2 = 0x30,
    CommCtrl = 0x31,
    GpioCtrl = 0x32,
    TiltFlipThreshLsb = 0x40,
    TiltFlipThreshMsb = 0x41,
    TiltFlipDebounce = 0x42,
    AnyMotionThreshLsb = 0x43,
    AnyMotionThreshMsb = 0x44,
    AnyMotionDebounce = 0x45,
    ShakeThreshLsb = 0x46,
    ShakeThreshMsb = 0x47,
    PeakToPeakDurationLsb = 0x48,
    PeakToPeakDurationMsb = 0x49,
    TimerControl = 0x4A,
    ReadCount = 0x4B,
}

impl From<Registers> for u8 {
    fn from(reg: Registers) -> Self {
        reg as u8
    }
}

// 0x05 DEV_STAT: Device Status

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum DevStatusState {
    Standby = 0x00,
    Wake = 0x01,
}

impl From<DevStatusState> for u8 {
    fn from(value: DevStatusState) -> Self {
        value as u8
    }
}

// 0x20 RANGE: Range Select Control

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum Range {
    Range2g = 0x00,
    Range4g = 0x01,
    Range8g = 0x02,
    Range16g = 0x03,
    Range12g = 0x04,
}

impl From<Range> for u8 {
    fn from(value: Range) -> Self {
        value as u8
    }
}

impl From<u8> for Range {
    fn from(value: u8) -> Self {
        // TODO: Maybe TryFrom?
        match value {
            0x00 => Range::Range2g,
            0x01 => Range::Range4g,
            0x02 => Range::Range8g,
            0x03 => Range::Range16g,
            0x04 => Range::Range12g,
            _ => Range::Range2g,
        }
    }
}

// TODO: These were originally un-annotated with int values; need to double-check.
#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum LpfBw {
    IdrBy4_255 = 0x00,
    IdrBy6 = 0x01,
    IdrBy12 = 0x02,
    IdrBy16 = 0x03,
}

impl From<LpfBw> for u8 {
    fn from(value: LpfBw) -> Self {
        value as u8
    }
}

impl From<u8> for LpfBw {
    fn from(value: u8) -> Self {
        // TODO: Maybe TryFrom?
        match value {
            0x00 => LpfBw::IdrBy4_255,
            0x01 => LpfBw::IdrBy6,
            0x02 => LpfBw::IdrBy12,
            0x03 => LpfBw::IdrBy16,
            _ => LpfBw::IdrBy4_255,
        }
    }
}

// 0x08 SR: Sample Rate
#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum SampleRate {
    Idr50Hz = 0x08,
    Idr100Hz = 0x09,
    Idr125Hz = 0x0A,
    Idr200Hz = 0x0B,
    Idr250Hz = 0x0C,
    Idr500Hz = 0x0D,
    Idr1000Hz = 0x0E,
    Idr2000Hz = 0x0F,
}

impl From<SampleRate> for u8 {
    fn from(value: SampleRate) -> Self {
        value as u8
    }
}

impl From<u8> for SampleRate {
    fn from(value: u8) -> Self {
        // TODO: Maybe TryFrom?
        match value {
            0x08 => SampleRate::Idr50Hz,
            0x09 => SampleRate::Idr100Hz,
            0x0A => SampleRate::Idr125Hz,
            0x0B => SampleRate::Idr200Hz,
            0x0C => SampleRate::Idr250Hz,
            0x0D => SampleRate::Idr500Hz,
            0x0E => SampleRate::Idr1000Hz,
            0x0F => SampleRate::Idr2000Hz,
            _ => SampleRate::Idr50Hz,
        }
    }
}

// 0x30 FIFO_CTRL2_SR2: FIFO Control Register 2, Sample Rate 2 Register
#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum DecMode {
    Disable = 0x00,
    DivideBy2 = 0x01,
    DivideBy4 = 0x02,
    DivideBy5 = 0x03,
    DivideBy8 = 0x04,
    DivideBy10 = 0x05,
    DivideBy16 = 0x06,
    DivideBy20 = 0x07,
    DivideBy40 = 0x08,
    DivideBy67 = 0x09,
    DivideBy80 = 0x0A,
    DivideBy100 = 0x0B,
    DivideBy200 = 0x0C,
    DivideBy250 = 0x0D,
    DivideBy500 = 0x0E,
    DivideBy1000 = 0x0F,
}

impl From<DecMode> for u8 {
    fn from(value: DecMode) -> Self {
        value as u8
    }
}

impl From<u8> for DecMode {
    fn from(value: u8) -> Self {
        // TODO: Maybe TryFrom?
        match value {
            0x00 => DecMode::Disable,
            0x01 => DecMode::DivideBy2,
            0x02 => DecMode::DivideBy4,
            0x03 => DecMode::DivideBy5,
            0x04 => DecMode::DivideBy8,
            0x05 => DecMode::DivideBy10,
            0x06 => DecMode::DivideBy16,
            0x07 => DecMode::DivideBy20,
            0x08 => DecMode::DivideBy40,
            0x09 => DecMode::DivideBy67,
            0x0A => DecMode::DivideBy80,
            0x0B => DecMode::DivideBy100,
            0x0C => DecMode::DivideBy200,
            0x0D => DecMode::DivideBy250,
            0x0E => DecMode::DivideBy500,
            0x0F => DecMode::DivideBy1000,
            _ => DecMode::Disable,
        }
    }
}

// 0x07 MODE: Mode
#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum ModeState {
    Sleep = 0x00,
    Wake = 0x01,
    Standby = 0x03,
}

impl From<ModeState> for u8 {
    fn from(value: ModeState) -> Self {
        value as u8
    }
}

impl From<u8> for ModeState {
    fn from(value: u8) -> Self {
        // TODO: Maybe TryFrom instead?
        match value {
            0x00 => ModeState::Sleep,
            0x01 => ModeState::Wake,
            0x03 => ModeState::Standby,
            _ => ModeState::Sleep,
        }
    }
}

// 0x4A TIMER_CTRL: Timer Control

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum Tilt35 {
    Duration1_6s = 0x00,
    Duration1_8s = 0x01,
    Duration2_0s = 0x02,
    Duration2_2s = 0x03,
    Duration2_4s = 0x04,
    Duration2_6s = 0x05,
    Duration2_8s = 0x06,
    Duration3_0s = 0x07,
}

impl From<Tilt35> for u8 {
    fn from(value: Tilt35) -> Self {
        value as u8
    }
}

impl From<f64> for Tilt35 {
    // Behavior is different here.
    // We're not treating this as a bitfield in the conversion, but as a numeric value.
    fn from(value: f64) -> Self {
        const VALUES: &[(f64, Tilt35)] = &[
            (1.6, Tilt35::Duration1_6s),
            (1.8, Tilt35::Duration1_8s),
            (2.0, Tilt35::Duration2_0s),
            (2.2, Tilt35::Duration2_2s),
            (2.4, Tilt35::Duration2_4s),
            (2.6, Tilt35::Duration2_6s),
            (2.8, Tilt35::Duration2_8s),
            (3.0, Tilt35::Duration3_0s),
        ];

        let mut nearest = VALUES[0].1;
        let mut min_diff = f64::MAX;

        for &(secs, period) in VALUES {
            let diff = if value > secs {
                value - secs
            } else {
                secs - value
            };
            if diff < min_diff {
                min_diff = diff;
                nearest = period;
            }
        }

        nearest
    }
}

impl From<u8> for Tilt35 {
    fn from(value: u8) -> Self {
        // TODO: Maybe TryFrom instead?
        match value {
            0x00 => Tilt35::Duration1_6s,
            0x01 => Tilt35::Duration1_8s,
            0x02 => Tilt35::Duration2_0s,
            0x03 => Tilt35::Duration2_2s,
            0x04 => Tilt35::Duration2_4s,
            0x05 => Tilt35::Duration2_6s,
            0x06 => Tilt35::Duration2_8s,
            0x07 => Tilt35::Duration3_0s,
            _ => Tilt35::Duration1_6s,
        }
    }
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum TempPeriod {
    Timeout200ms = 0x00,
    Timeout400ms = 0x01,
    Timeout800ms = 0x02,
    Timeout1600ms = 0x03,
    Timeout3200ms = 0x04,
    Timeout6400ms = 0x05,
}

impl From<TempPeriod> for u8 {
    fn from(value: TempPeriod) -> Self {
        value as u8
    }
}

impl From<u16> for TempPeriod {
    // Behavior is different here.
    // We're not treating this as a bitfield in the conversion, but as a numeric value.
    fn from(value: u16) -> Self {
        const VALUES: &[(u16, TempPeriod)] = &[
            (200, TempPeriod::Timeout200ms),
            (400, TempPeriod::Timeout400ms),
            (800, TempPeriod::Timeout800ms),
            (1600, TempPeriod::Timeout1600ms),
            (3200, TempPeriod::Timeout3200ms),
            (6400, TempPeriod::Timeout6400ms),
        ];

        let mut nearest = VALUES[0].1;
        let mut min_diff = u16::MAX;

        for &(ms, period) in VALUES {
            let diff = if value > ms { value - ms } else { ms - value };
            if diff < min_diff {
                min_diff = diff;
                nearest = period;
            }
        }

        nearest
    }
}
