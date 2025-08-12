use crate::platform::bq25895::registers::Register;

#[derive(Clone, Copy)]
pub struct Reg07 {
    pub en_term: bool,
    pub stat_dis: bool,
    pub watchdog: WatchdogTimerSetting,
    pub en_timer: bool,
    pub chg_timer: FastChargeTimeSetting,
}

impl Reg07 {
    pub fn new(
        en_term: bool,
        stat_dis: bool,
        watchdog: WatchdogTimerSetting,
        en_timer: bool,
        chg_timer: FastChargeTimeSetting,
    ) -> Self {
        Self {
            en_term,
            stat_dis,
            watchdog,
            en_timer,
            chg_timer,
        }
    }
}

impl Default for Reg07 {
    fn default() -> Self {
        Self {
            en_term: true,
            stat_dis: false,
            watchdog: WatchdogTimerSetting::default(),
            en_timer: true,
            chg_timer: FastChargeTimeSetting::default(),
        }
    }
}

impl Register for Reg07 {
    const ADDRESS: u8 = 0x07;
}

impl From<u8> for Reg07 {
    fn from(byte: u8) -> Self {
        Self {
            en_term: byte & 0x80 != 0,
            stat_dis: byte & 0x40 != 0,
            watchdog: (byte >> 4 & 0x03).into(),
            en_timer: byte & 0x08 != 0,
            chg_timer: (byte >> 1 & 0x03).into(),
        }
    }
}

impl From<&Reg07> for u8 {
    fn from(reg: &Reg07) -> Self {
        let mut byte = 0;
        byte |= (reg.en_term as u8) << 7;
        byte |= (reg.stat_dis as u8) << 6;
        byte |= u8::from(reg.watchdog) << 4;
        byte |= (reg.en_timer as u8) << 3;
        byte |= u8::from(reg.chg_timer) << 1;
        byte |= 0x01;
        byte
    }
}

impl From<Reg07> for u8 {
    fn from(reg: Reg07) -> Self {
        u8::from(&reg)
    }
}

impl core::fmt::Display for Reg07 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Reg07 {{ {:#08b}: EnTerm={}, StatDis={}, EnTimer={}, Watchdog={}, ChgTimer={} }}",
            u8::from(self),
            self.en_term,
            self.stat_dis,
            self.en_timer,
            self.watchdog,
            self.chg_timer,
        )
    }
}

impl defmt::Format for Reg07 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg07 {{ {:#08b}: EnTerm={}, StatDis={}, EnTimer={}, Watchdog={}, ChgTimer={} }}",
            u8::from(self),
            self.en_term,
            self.stat_dis,
            self.en_timer,
            self.watchdog,
            self.chg_timer,
        )
    }
}

#[derive(Clone, Copy)]
pub enum TReg {
    Setting60deg = 0x00,
    Setting80deg = 0x01,
    Setting100deg = 0x02,
    Setting120deg = 0x03,
}

impl Default for TReg {
    fn default() -> Self {
        TReg::Setting120deg
    }
}

impl From<TReg> for u8 {
    fn from(value: TReg) -> Self {
        value.into()
    }
}

impl From<u8> for TReg {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => TReg::Setting60deg,
            0x01 => TReg::Setting80deg,
            0x02 => TReg::Setting100deg,
            _ => TReg::Setting120deg,
        }
    }
}

impl core::fmt::Display for TReg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "TReg {{ {:b}: {} }}",
            *self as u8,
            match self {
                TReg::Setting60deg => "60deg",
                TReg::Setting80deg => "80deg",
                TReg::Setting100deg => "100deg",
                TReg::Setting120deg => "120deg",
            }
        )
    }
}

impl defmt::Format for TReg {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "TReg {{ {:b}: {} }}",
            *self as u8,
            match self {
                TReg::Setting60deg => "60deg",
                TReg::Setting80deg => "80deg",
                TReg::Setting100deg => "100deg",
                TReg::Setting120deg => "120deg",
            }
        )
    }
}

#[derive(Clone, Copy)]
pub enum WatchdogTimerSetting {
    Disabled = 0x00,
    Enabled40s = 0x01,
    Enabled80s = 0x02,
    Enabled160s = 0x03,
}

impl Default for WatchdogTimerSetting {
    fn default() -> Self {
        WatchdogTimerSetting::Enabled40s
    }
}

impl From<WatchdogTimerSetting> for u8 {
    fn from(value: WatchdogTimerSetting) -> Self {
        match value {
            WatchdogTimerSetting::Disabled => 0x00,
            WatchdogTimerSetting::Enabled40s => 0x01,
            WatchdogTimerSetting::Enabled80s => 0x02,
            WatchdogTimerSetting::Enabled160s => 0x03,
        }
    }
}

impl From<u8> for WatchdogTimerSetting {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => WatchdogTimerSetting::Disabled,
            0x01 => WatchdogTimerSetting::Enabled40s,
            0x02 => WatchdogTimerSetting::Enabled80s,
            0x03 => WatchdogTimerSetting::Enabled160s,
            _ => WatchdogTimerSetting::Disabled,
        }
    }
}

impl core::fmt::Display for WatchdogTimerSetting {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "WatchdogTimerSetting {{ 0b{:b}: {} }}",
            *self as u8,
            match self {
                WatchdogTimerSetting::Disabled => "Disabled",
                WatchdogTimerSetting::Enabled40s => "40s",
                WatchdogTimerSetting::Enabled80s => "80s",
                WatchdogTimerSetting::Enabled160s => "160s",
            },
        )
    }
}

impl defmt::Format for WatchdogTimerSetting {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "WatchdogTimerSetting {{ 0b{:b}: {} }}",
            *self as u8,
            match self {
                WatchdogTimerSetting::Disabled => "Disabled",
                WatchdogTimerSetting::Enabled40s => "40s",
                WatchdogTimerSetting::Enabled80s => "80s",
                WatchdogTimerSetting::Enabled160s => "160s",
            },
        )
    }
}

#[derive(Clone, Copy)]
pub enum FastChargeTimeSetting {
    Setting5h = 0x00,
    Setting8h = 0x01,
    Setting12h = 0x02,
    Setting20h = 0x03,
}

impl Default for FastChargeTimeSetting {
    fn default() -> Self {
        FastChargeTimeSetting::Setting12h
    }
}

impl From<FastChargeTimeSetting> for u8 {
    fn from(value: FastChargeTimeSetting) -> Self {
        match value {
            FastChargeTimeSetting::Setting5h => 0x00,
            FastChargeTimeSetting::Setting8h => 0x01,
            FastChargeTimeSetting::Setting12h => 0x02,
            FastChargeTimeSetting::Setting20h => 0x03,
        }
    }
}

impl From<u8> for FastChargeTimeSetting {
    fn from(value: u8) -> Self {
        match value {
            0x00 => FastChargeTimeSetting::Setting5h,
            0x01 => FastChargeTimeSetting::Setting8h,
            0x02 => FastChargeTimeSetting::Setting12h,
            _ => FastChargeTimeSetting::Setting20h,
        }
    }
}

impl core::fmt::Display for FastChargeTimeSetting {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "FastChargeTimeSetting {{ 0b{:b}: {} }}",
            *self as u8,
            match self {
                FastChargeTimeSetting::Setting5h => "5h",
                FastChargeTimeSetting::Setting8h => "8h",
                FastChargeTimeSetting::Setting12h => "12h",
                FastChargeTimeSetting::Setting20h => "20h",
            }
        )
    }
}

impl defmt::Format for FastChargeTimeSetting {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "FastChargeTimeSetting {{ 0b{:b}: {} }}",
            *self as u8,
            match self {
                FastChargeTimeSetting::Setting5h => "5h",
                FastChargeTimeSetting::Setting8h => "8h",
                FastChargeTimeSetting::Setting12h => "12h",
                FastChargeTimeSetting::Setting20h => "20h",
            }
        )
    }
}
