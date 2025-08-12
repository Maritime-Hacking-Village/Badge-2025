pub mod cpu_ctrl;

pub trait Register {
    const ADDRESS: u16;
}
