use super::irqs::Irqs;
use embassy_rp::{
    dma::Channel,
    peripherals::PIO0,
    pio::{Pio, PioPin},
    pio_programs::ws2812::{PioWs2812, PioWs2812Program},
    Peri,
};

// NOTE: 5 extra "LEDs" to hold the flag.
pub const NUM_LEDS: usize = 9 + 5;

pub fn setup(
    pio: Peri<'static, PIO0>,
    dma: Peri<'static, impl Channel>,
    pin: Peri<'static, impl PioPin>,
) -> PioWs2812<'static, PIO0, 0, NUM_LEDS> {
    let Pio {
        mut common, sm0, ..
    } = Pio::new(pio, Irqs);
    let program = PioWs2812Program::new(&mut common);

    PioWs2812::new(&mut common, sm0, dma, pin, &program)
}
