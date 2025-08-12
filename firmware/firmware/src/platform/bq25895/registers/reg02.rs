use crate::platform::{bq25895::registers::Register, util::bool_array_to_u8};

#[derive(Clone, Copy)]
pub struct Reg02 {
    pub conv_start: bool,
    pub conv_rate: bool,
    pub boost_freq: bool,
    pub ico_en: bool,
    pub hvdcp_en: bool,
    pub maxc_en: bool,
    pub force_dpdm: bool,
    pub auto_dpdm_en: bool,
}

impl Reg02 {
    pub fn new(
        conv_start: bool,
        conv_rate: bool,
        boost_freq: bool,
        ico_en: bool,
        hvdcp_en: bool,
        maxc_en: bool,
        force_dpdm: bool,
        auto_dpdm_en: bool,
    ) -> Self {
        Self {
            conv_start,
            conv_rate,
            boost_freq,
            ico_en,
            hvdcp_en,
            maxc_en,
            force_dpdm,
            auto_dpdm_en,
        }
    }
}

impl Default for Reg02 {
    fn default() -> Self {
        Self {
            conv_start: false,
            conv_rate: false,
            boost_freq: true,
            ico_en: true,
            hvdcp_en: true,
            maxc_en: true,
            force_dpdm: false,
            auto_dpdm_en: true,
        }
    }
}

impl Register for Reg02 {
    const ADDRESS: u8 = 0x02;
}

impl From<u8> for Reg02 {
    fn from(b: u8) -> Self {
        Reg02 {
            conv_start: (b & 0x80) != 0,
            conv_rate: (b & 0x40) != 0,
            boost_freq: (b & 0x20) != 0,
            ico_en: (b & 0x10) != 0,
            hvdcp_en: (b & 0x08) != 0,
            maxc_en: (b & 0x04) != 0,
            force_dpdm: (b & 0x02) != 0,
            auto_dpdm_en: (b & 0x01) != 0,
        }
    }
}

impl From<&Reg02> for u8 {
    fn from(reg: &Reg02) -> Self {
        bool_array_to_u8(&[
            reg.conv_start,
            reg.conv_rate,
            reg.boost_freq,
            reg.ico_en,
            reg.hvdcp_en,
            reg.maxc_en,
            reg.force_dpdm,
            reg.auto_dpdm_en,
        ])
    }
}

impl From<Reg02> for u8 {
    fn from(reg: Reg02) -> Self {
        u8::from(&reg)
    }
}

impl core::fmt::Display for Reg02 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Reg02 {{ {:#08b}: ConvStart={}, ConvRate={}, BoostFreq={}, ICOEn={}, HVDCPEn={}, MaxCEn={}, ForceDPDM={}, AutoDPDMEn={} }}",
            u8::from(self),
            self.conv_start,
            match self.conv_rate {
                true => "Continuous1s",
                false => "OneShot",
            },
            self.boost_freq,
            self.ico_en,
            self.hvdcp_en,
            self.maxc_en,
            self.force_dpdm,
            self.auto_dpdm_en,
        )
    }
}

impl defmt::Format for Reg02 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "Reg02 {{ {:#08b}: ConvStart={}, ConvRate={}, BoostFreq={}, ICOEn={}, HVDCPEn={}, MaxCEn={}, ForceDPDM={}, AutoDPDMEn={} }}",
            u8::from(self),
            self.conv_start,
            match self.conv_rate {
                true => "Continuous1s",
                false => "OneShot",
            },
            self.boost_freq,
            self.ico_en,
            self.hvdcp_en,
            self.maxc_en,
            self.force_dpdm,
            self.auto_dpdm_en,
        )
    }
}
