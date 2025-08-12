use crate::platform::bq25895::registers::Register;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reg0b {
    pub vbus_stat: VbusStatus,
    pub chrg_stat: ChargingStatus,
    pub pg_stat: bool,
    pub sdp_stat: bool,
    pub vsys_stat: bool,
}

impl Reg0b {
    pub fn new(
        vbus_stat: VbusStatus,
        chrg_stat: ChargingStatus,
        pg_stat: bool,
        sdp_stat: bool,
        vsys_stat: bool,
    ) -> Self {
        Self {
            vbus_stat,
            chrg_stat,
            pg_stat,
            sdp_stat,
            vsys_stat,
        }
    }
}

impl Register for Reg0b {
    const ADDRESS: u8 = 0x0b;
}

impl From<u8> for Reg0b {
    fn from(byte: u8) -> Self {
        Self {
            vbus_stat: VbusStatus::from_byte(byte >> 5 & 0x07),
            chrg_stat: ChargingStatus::from_byte(byte >> 3 & 0x03),
            pg_stat: byte & 0x04 != 0,
            sdp_stat: byte & 0x02 != 0,
            vsys_stat: byte & 0x01 != 0,
        }
    }
}

impl From<&Reg0b> for u8 {
    fn from(reg: &Reg0b) -> Self {
        let mut byte = 0;
        byte |= (reg.vbus_stat as u8) << 5;
        byte |= (reg.chrg_stat as u8) << 3;
        byte |= (reg.pg_stat as u8) << 2;
        byte |= (reg.sdp_stat as u8) << 1;
        byte |= reg.vsys_stat as u8;
        byte
    }
}

impl From<Reg0b> for u8 {
    fn from(reg: Reg0b) -> Self {
        u8::from(&reg)
    }
}

impl core::fmt::Display for Reg0b {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Reg0b {{ {:#08b}: VbusStat={}, ChrgStat={}, PgStat={}, SdpStat={}, VsysStat={} }}",
            u8::from(self),
            self.vbus_stat,
            self.chrg_stat,
            self.pg_stat,
            self.sdp_stat,
            self.vsys_stat
        )
    }
}

impl defmt::Format for Reg0b {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg0b {{ {:#08b}: VbusStat={}, ChrgStat={}, PgStat={}, SdpStat={}, VsysStat={} }}",
            u8::from(self),
            self.vbus_stat,
            self.chrg_stat,
            self.pg_stat,
            self.sdp_stat,
            self.vsys_stat
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VbusStatus {
    #[default]
    NoInput = 0x00,
    UsbHostSDP = 0x01,
    UsbCDP = 0x02,
    UsbDCP = 0x03,
    AdjustableHighVoltageDCP = 0x04,
    UnknownAdapter500mA = 0x05,
    NonStandardAdapter = 0x06,
    OTG = 0x07,
}

impl VbusStatus {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => VbusStatus::NoInput,
            0x01 => VbusStatus::UsbHostSDP,
            0x02 => VbusStatus::UsbCDP,
            0x03 => VbusStatus::UsbDCP,
            0x04 => VbusStatus::AdjustableHighVoltageDCP,
            0x05 => VbusStatus::UnknownAdapter500mA,
            0x06 => VbusStatus::NonStandardAdapter,
            0x07 => VbusStatus::OTG,
            _ => VbusStatus::NoInput,
        }
    }
}

impl From<VbusStatus> for u8 {
    fn from(value: VbusStatus) -> Self {
        value.into()
    }
}

impl From<u8> for VbusStatus {
    fn from(value: u8) -> Self {
        value.into()
    }
}

impl core::fmt::Display for VbusStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "VbusStatus {{ {:b}: {} }}",
            self.clone() as u8,
            match self {
                VbusStatus::NoInput => "NoInput",
                VbusStatus::UsbHostSDP => "UsbHostSDP",
                VbusStatus::UsbCDP => "UsbCDP",
                VbusStatus::UsbDCP => "UsbDCP",
                VbusStatus::AdjustableHighVoltageDCP => "AdjustableHighVoltageDCP",
                VbusStatus::UnknownAdapter500mA => "UnknownAdapter500mA",
                VbusStatus::NonStandardAdapter => "NonStandardAdapter",
                VbusStatus::OTG => "OTG",
            }
        )
    }
}

impl defmt::Format for VbusStatus {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "VbusStatus {{ {:b}: {} }}",
            self.clone() as u8,
            match self {
                VbusStatus::NoInput => "NoInput",
                VbusStatus::UsbHostSDP => "UsbHostSDP",
                VbusStatus::UsbCDP => "UsbCDP",
                VbusStatus::UsbDCP => "UsbDCP",
                VbusStatus::AdjustableHighVoltageDCP => "AdjustableHighVoltageDCP",
                VbusStatus::UnknownAdapter500mA => "UnknownAdapter500mA",
                VbusStatus::NonStandardAdapter => "NonStandardAdapter",
                VbusStatus::OTG => "OTG",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ChargingStatus {
    #[default]
    NotCharging = 0x00,
    PreCharge = 0x01,
    FastCharging = 0x02,
    ChargeTerminationDone = 0x03,
}

impl ChargingStatus {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => ChargingStatus::NotCharging,
            0x01 => ChargingStatus::PreCharge,
            0x02 => ChargingStatus::FastCharging,
            0x03 => ChargingStatus::ChargeTerminationDone,
            _ => ChargingStatus::NotCharging,
        }
    }
}

impl From<ChargingStatus> for u8 {
    fn from(value: ChargingStatus) -> Self {
        value.into()
    }
}

impl From<u8> for ChargingStatus {
    fn from(value: u8) -> Self {
        value.into()
    }
}

impl core::fmt::Display for ChargingStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ChargingStatus {{ {:b}: {} }}",
            self.clone() as u8,
            match self {
                ChargingStatus::NotCharging => "Not Charging",
                ChargingStatus::PreCharge => "Pre-Charge",
                ChargingStatus::FastCharging => "Fast Charging",
                ChargingStatus::ChargeTerminationDone => "Charge Termination Done",
            },
        )
    }
}

impl defmt::Format for ChargingStatus {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "ChargingStatus {{ {:b}: {} }}",
            self.clone() as u8,
            match self {
                ChargingStatus::NotCharging => "Not Charging",
                ChargingStatus::PreCharge => "Pre-Charge",
                ChargingStatus::FastCharging => "Fast Charging",
                ChargingStatus::ChargeTerminationDone => "Charge Termination Done",
            },
        )
    }
}
