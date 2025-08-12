use crate::platform::bq25895::registers::Register;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg0c {
    pub watchdog_fault: bool,
    pub boost_fault: bool,
    pub chrg_fault: ChargeFaultStatus,
    pub bat_fault: bool,
    pub ntc_fault: NTCFaultStatus,
}

impl Reg0c {
    pub fn new(
        watchdog_fault: bool,
        boost_fault: bool,
        chrg_fault: ChargeFaultStatus,
        bat_fault: bool,
        ntc_fault: NTCFaultStatus,
    ) -> Self {
        Self {
            watchdog_fault,
            boost_fault,
            chrg_fault,
            bat_fault,
            ntc_fault,
        }
    }
}

impl Register for Reg0c {
    const ADDRESS: u8 = 0x0c;
}

impl From<u8> for Reg0c {
    fn from(byte: u8) -> Self {
        Self {
            watchdog_fault: byte & 0x80 != 0,
            boost_fault: byte & 0x40 != 0,
            chrg_fault: ChargeFaultStatus::from_byte(byte >> 4 & 0x03),
            bat_fault: byte & 0x08 != 0,
            ntc_fault: NTCFaultStatus::from_byte(byte & 0x07),
        }
    }
}

impl From<&Reg0c> for u8 {
    fn from(reg: &Reg0c) -> Self {
        let mut byte = 0;
        byte |= (reg.watchdog_fault as u8) << 7;
        byte |= (reg.boost_fault as u8) << 6;
        byte |= (reg.chrg_fault as u8) << 5;
        byte |= (reg.bat_fault as u8) << 3;
        byte |= reg.ntc_fault as u8;
        byte
    }
}

impl From<Reg0c> for u8 {
    fn from(reg: Reg0c) -> Self {
        u8::from(&reg)
    }
}

impl core::fmt::Display for Reg0c {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Reg0c {{ {:#08b}: WatchdogFault={}, BoostFault={}, ChrgFault={}, BatFault={}, NtcFault={} }}",
            u8::from(self),
            self.watchdog_fault,
            self.boost_fault,
            self.chrg_fault,
            self.bat_fault,
            self.ntc_fault
        )
    }
}

impl defmt::Format for Reg0c {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg0c {{ {:#08b}: WatchdogFault={}, BoostFault={}, ChrgFault={}, BatFault={}, NtcFault={} }}",
            u8::from(self),
            self.watchdog_fault,
            self.boost_fault,
            self.chrg_fault,
            self.bat_fault,
            self.ntc_fault
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ChargeFaultStatus {
    #[default]
    Normal = 0x00,
    InputFault = 0x01,
    ThermalShutdown = 0x02,
    ChargeSafetyTimerExpiration = 0x03,
}

impl ChargeFaultStatus {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => ChargeFaultStatus::Normal,
            0x01 => ChargeFaultStatus::InputFault,
            0x02 => ChargeFaultStatus::ThermalShutdown,
            0x03 => ChargeFaultStatus::ChargeSafetyTimerExpiration,
            _ => ChargeFaultStatus::Normal,
        }
    }
}

impl From<ChargeFaultStatus> for u8 {
    fn from(value: ChargeFaultStatus) -> Self {
        value.into()
    }
}

impl From<u8> for ChargeFaultStatus {
    fn from(value: u8) -> Self {
        value.into()
    }
}

impl core::fmt::Display for ChargeFaultStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ChargeFaultStatus {{ 0b{:b}: {} }}",
            self.clone() as u8,
            match self {
                ChargeFaultStatus::Normal => "Normal",
                ChargeFaultStatus::InputFault => "Input Fault",
                ChargeFaultStatus::ThermalShutdown => "Thermal Shutdown",
                ChargeFaultStatus::ChargeSafetyTimerExpiration => "Charge Safety Timer Expiration",
            }
        )
    }
}

impl defmt::Format for ChargeFaultStatus {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "ChargeFaultStatus {{ 0b{:b}: {} }}",
            self.clone() as u8,
            match self {
                ChargeFaultStatus::Normal => "Normal",
                ChargeFaultStatus::InputFault => "Input Fault",
                ChargeFaultStatus::ThermalShutdown => "Thermal Shutdown",
                ChargeFaultStatus::ChargeSafetyTimerExpiration => "Charge Safety Timer Expiration",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NTCFaultStatus {
    #[default]
    Normal = 0x00,
    TsCold = 0x01,
    TsHot = 0x02,
}

impl NTCFaultStatus {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => NTCFaultStatus::Normal,
            0x01 => NTCFaultStatus::TsCold,
            0x02 => NTCFaultStatus::TsHot,
            _ => NTCFaultStatus::Normal,
        }
    }
}

impl From<NTCFaultStatus> for u8 {
    fn from(value: NTCFaultStatus) -> Self {
        value.into()
    }
}

impl From<u8> for NTCFaultStatus {
    fn from(value: u8) -> Self {
        value.into()
    }
}

impl core::fmt::Display for NTCFaultStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "NTCFaultStatus {{ 0b{:b}: {} }}",
            self.clone() as u8,
            match self {
                NTCFaultStatus::Normal => "Normal",
                NTCFaultStatus::TsCold => "TS Cold",
                NTCFaultStatus::TsHot => "TS Hot",
            },
        )
    }
}

impl defmt::Format for NTCFaultStatus {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "NTCFaultStatus {{ 0b{:b}: {} }}",
            self.clone() as u8,
            match self {
                NTCFaultStatus::Normal => "Normal",
                NTCFaultStatus::TsCold => "TS Cold",
                NTCFaultStatus::TsHot => "TS Hot",
            },
        )
    }
}
