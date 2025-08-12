use super::irqs::Irqs;
use cyw43::{Control, NetDriver, Runner, State};
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use embassy_rp::{
    dma::Channel,
    gpio::{Level, Output, Pin},
    peripherals::PIO1,
    pio::{Pio, PioPin},
    Peri,
};
use static_cell::StaticCell;

pub async fn setup<DMA: Channel>(
    pwr: Peri<'static, impl Pin>,
    cs: Peri<'static, impl Pin>,
    dio: Peri<'static, impl PioPin>,
    clk: Peri<'static, impl PioPin>,
    pio: Peri<'static, PIO1>,
    dma: Peri<'static, DMA>,
) -> (
    NetDriver<'static>,
    Control<'static>,
    Runner<'static, Output<'static>, PioSpi<'static, PIO1, 0, DMA>>,
) {
    let fw = include_bytes!("../../cyw43-firmware/43439A0.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download ../../embassy/cyw43-firmware/43439A0.bin --binary-format bin --chip RP235X --base-address 0x10200000
    // let fw = unsafe { core::slice::from_raw_parts(0x10200000 as *const u8, 230321) };

    let pwr = Output::new(pwr, Level::Low);
    let cs = Output::new(cs, Level::High);
    let mut pio = Pio::new(pio, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        dio,
        clk,
        dma,
    );

    static STATE: StaticCell<State> = StaticCell::new();
    let state = STATE.init(State::new());

    cyw43::new(state, pwr, spi, fw).await
}
