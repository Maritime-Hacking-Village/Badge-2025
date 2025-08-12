use defmt::Format;

pub mod reg00;
pub mod reg01;
pub mod reg02;
pub mod reg03;
pub mod reg04;
pub mod reg05;
pub mod reg06;
pub mod reg07;
pub mod reg08;
pub mod reg09;
pub mod reg0a;
pub mod reg0b;
pub mod reg0c;
pub mod reg0d;
pub mod reg0e;
pub mod reg0f;
pub mod reg10;
pub mod reg11;
pub mod reg12;
pub mod reg13;
pub mod reg14;

#[derive(Debug, Format, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatusRegisters {
    pub reg0b: reg0b::Reg0b,
    pub reg0c: reg0c::Reg0c,
    pub reg0e: reg0e::Reg0e,
    pub reg0f: reg0f::Reg0f,
    pub reg10: reg10::Reg10,
    pub reg11: reg11::Reg11,
    pub reg12: reg12::Reg12,
    pub reg13: reg13::Reg13,
}

pub trait Register: From<u8> + Into<u8> + Clone {
    const ADDRESS: u8;
}
