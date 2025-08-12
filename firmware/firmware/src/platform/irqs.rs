use embassy_rp::{
    bind_interrupts, i2c,
    peripherals::{I2C0, PIO0, PIO1, PIO2, TRNG, UART1, USB},
    pio, uart, usb,
};

bind_interrupts!(pub struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    UART1_IRQ => uart::InterruptHandler<UART1>;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    // PIO0_IRQ_1 => pio::InterruptHandler<PIO0>;
    PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
    PIO2_IRQ_0 => pio::InterruptHandler<PIO2>;
    TRNG_IRQ => embassy_rp::trng::InterruptHandler<TRNG>;
});
