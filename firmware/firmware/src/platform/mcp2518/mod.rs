pub mod registers;

// pub trait Register {
//     const ADDRESS: u8;
// }

pub struct OSC {
    pub sclkrdy: bool,
    pub oscrdy: bool,
    pub pllrdy: bool,
    pub clkodiv: [bool; 2],
    pub sclkdiv: bool,
    pub lpmem: bool,
    pub oscdis: bool,
    pub pllen: bool,
}
