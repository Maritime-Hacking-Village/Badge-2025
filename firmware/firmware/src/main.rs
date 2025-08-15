#![no_std]
#![no_main]

extern crate alloc;

mod apps;
mod platform;
mod tasks;

#[cfg(feature = "heap-in-psram")]
use crate::platform::psram;
use crate::{
    apps::{neopixel::neopixel_task, usb_cli},
    platform::{
        async_io_on_sync_io::AsyncOutputPin,
        bq25895::{self},
        i2c_io_expander::{
            self,
            models::{pca9536::PCA9536, tcal9539::TCAL9539},
        },
        interrupt_i2c::{self},
        irqs::Irqs,
        mc3479,
        multi_write::MultiWrite,
        neopixel::{self},
        repl::{
            self,
            console::{ConsolePipe, CONSOLE_PIPE},
        },
        shared_spi_bus::SharedSpiBusWithConfig,
        usb,
        usb_cdc_io::UsbCdcIo,
    },
};
use alloc::vec::Vec;
use apps::logging::Logger;
use defmt::{debug, unwrap, warn};
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_executor::{Executor, Spawner};
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    i2c::{self},
    multicore::Stack,
    peripherals::{I2C0, SPI0},
    spi::{self, Spi},
    trng::Trng,
};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    mutex::Mutex,
};
use embassy_time::{Delay, Timer};
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::{RgbColor, *},
};
use mipidsi::{interface::SpiInterfaceAsync, models::ST7789, options::ColorInversion};
use panic_probe as _;
use static_cell::StaticCell;

#[global_allocator]
pub static HEAP: Heap = Heap::empty();
static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) -> () {
    let p = embassy_rp::init(Default::default());

    #[cfg(not(feature = "heap-in-psram"))]
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024 * 210;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    // Initialize PSRAM before logging (for heap)
    #[cfg(feature = "heap-in-psram")]
    {
        let size = psram::init();
        assert!(size > 0);
        unsafe { HEAP.init(0x11000000 as usize, size) }
        info!("Heap initialized on external PSRAM!");
    }

    // RPC channels
    let (app_watch, app_ack_channel) = make_app_channels!();
    let (call_channel, result_channel) = make_rpc_channels!();
    let (repl_in_channel, repl_out_channel) = make_repl_channels!();

    #[allow(static_mut_refs)]
    embassy_rp::multicore::spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner| {
            spawner
                .spawn(tasks::repl::repl_task(
                    call_channel.sender(),
                    result_channel.receiver(),
                    repl_in_channel.receiver(),
                    repl_out_channel.sender(),
                ))
                .unwrap()
        });
    });

    // USB
    let (usb, logger, cli, _storage) = usb::initialize(p.USB);

    // SD Card and Display
    const DISPLAY_FREQ: u32 = 62_500_000;
    const SD_FREQ: u32 = 400_000;

    let mut lcd_config = spi::Config::default();
    lcd_config.frequency = DISPLAY_FREQ;
    lcd_config.phase = spi::Phase::CaptureOnFirstTransition;
    lcd_config.polarity = spi::Polarity::IdleLow;

    let mut sd_config = spi::Config::default();
    sd_config.frequency = SD_FREQ;
    sd_config.phase = spi::Phase::CaptureOnFirstTransition;
    sd_config.polarity = spi::Polarity::IdleLow;

    static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spi<'static, SPI0, spi::Async>>> =
        StaticCell::new();
    let spi_bus = Mutex::new(Spi::new(
        p.SPI0,
        p.PIN_2,
        p.PIN_3,
        p.PIN_4,
        p.DMA_CH2,
        p.DMA_CH3,
        sd_config.clone(),
    ));
    let spi_bus = SPI_BUS.init(spi_bus);

    let sd_bus = SharedSpiBusWithConfig::new(spi_bus, sd_config);
    let sd_cs_out = AsyncOutputPin(embassy_rp::gpio::Output::new(
        p.PIN_14,
        embassy_rp::gpio::Level::High,
    ));
    // let mut sd_card = SdCard::new(sd_bus, sd_cs_out);
    // TODO: run this when SD card is inserted
    // sdcard::bus::init(&mut sd_card).await.unwrap();

    // Display
    let lcd_spi = SpiDeviceWithConfig::new(spi_bus, Output::new(p.PIN_8, Level::High), lcd_config);
    let dcx = Output::new(p.PIN_5, Level::Low);
    // let lcd_rst = Output::new(p.PIN_25, Level::Low);
    let lcd_interface = SpiInterfaceAsync::new(lcd_spi, dcx);
    // let _bl = Output::new(p.PIN_9, Level::High);

    // I2C devices
    static I2C_BUS: StaticCell<Mutex<CriticalSectionRawMutex, i2c::I2c<'_, I2C0, i2c::Async>>> =
        StaticCell::new();

    let i2c_config = embassy_rp::i2c::Config::default();
    let bus = embassy_rp::i2c::I2c::new_async(p.I2C0, p.PIN_1, p.PIN_0, Irqs, i2c_config);
    let i2c_bus = I2C_BUS.init(Mutex::new(bus));

    // Accelerometer
    let (shake_watch, shake_signal_ready, shake_signal_done) = make_shake_channels!();
    let (mut accel_ctrl, accel_runner) = mc3479::setup(i2c_bus, shake_watch.sender());

    // IRQ pin
    let int_pin = Input::new(p.PIN_15, Pull::None);

    // Display
    const LCD_BUF_SIZE: usize = 205 * 320 * 2;
    static LCD_BUFFER: StaticCell<[u8; LCD_BUF_SIZE]> = StaticCell::new();
    let lcd_buffer = LCD_BUFFER.init([0u8; LCD_BUF_SIZE]);

    let mut display = mipidsi::Builder::new_async(ST7789, lcd_interface)
        .display_size(205, 320)
        .invert_colors(ColorInversion::Inverted)
        // .reset_pin(lcd_rst)
        .init_async(&mut Delay, lcd_buffer)
        .await
        .unwrap();

    // Clear and enable LCD backlight
    display.clear(Rgb565::BLACK).unwrap();
    display.flush().await.unwrap();

    // Anonymous scope to free once done rendering.
    {
        let splash =
            tinygif::Gif::<Rgb565>::from_slice(include_bytes!("../assets/splash.gif")).unwrap();
        let logo =
            tinygif::Gif::<Rgb565>::from_slice(include_bytes!("../assets/logo.gif")).unwrap();

        for frame in splash.frames() {
            // let start = Instant::now();
            frame
                .draw(&mut display.translated(Point::new(35, 0)))
                .unwrap();
            display.flush().await.unwrap();
            // let remain_delay =
            //     ((frame.delay_centis as u64) * 10).saturating_sub(start.elapsed().as_millis());
            // debug!("{}", remain_delay);
            // Timer::after_millis(remain_delay).await;
        }

        display.clear(Rgb565::BLACK).unwrap();
        display.flush().await.unwrap();

        for frame in logo.frames() {
            // let start = Instant::now();
            frame
                .draw(&mut display.translated(Point::new(35, 0)))
                .unwrap();
            display.flush().await.unwrap();
            // let remain_delay =
            //     ((frame.delay_centis as u64) * 10).saturating_sub(start.elapsed().as_millis());
            // Timer::after_millis(remain_delay).await;
        }
    }

    let display_channel = make_display_channel!();
    let console_pipe = CONSOLE_PIPE.init(ConsolePipe::new());
    let (console_reader, console_writer) = console_pipe.split();

    unwrap!(spawner.spawn(crate::apps::display::display_task(
        unwrap!(app_watch.receiver()),
        app_ack_channel.sender(),
        console_reader,
        display_channel.receiver(),
        display,
        unwrap!(shake_watch.receiver()),
        shake_signal_ready,
        shake_signal_done
    )));

    // NeoPixel
    let ws2812 = neopixel::setup(p.PIO0, p.DMA_CH0, p.PIN_17);
    let led_channel = make_leds_channel!();

    // Run the USB device.
    unwrap!(spawner.spawn(usb::task(usb)));

    // Initialize logging before printing
    let usb_log = UsbCdcIo(logger);
    let multi_writer = MultiWrite::new(usb_log, console_writer);
    let logger = Logger::new(multi_writer);
    unwrap!(spawner.spawn(tasks::log::log_task(logger)));

    // Wifi
    #[cfg(feature = "wifi")]
    {
        use crate::{
            apps::wifi_tcp_cli,
            platform::{net_stack, wifi},
        };

        // Wifi device
        let (net_device, mut cyw43_control, cyw43_runner) =
            wifi::setup(p.PIN_23, p.PIN_25, p.PIN_24, p.PIN_29, p.PIO1, p.DMA_CH1).await;
        unwrap!(spawner.spawn(tasks::wifi::cyw43_task(cyw43_runner)));

        // Network Stack
        let (net_stack, net_runner) = net_stack::setup(&mut cyw43_control, net_device).await;
        unwrap!(spawner.spawn(net_stack::net_task(net_runner)));

        // Network App
        unwrap!(spawner.spawn(wifi_tcp_cli::cli_task(cyw43_control, net_stack)));
    };

    // Gpio Expander 1 (TCAL9539)
    let (gpio_exp_1_ctrl_1, gpio_exp_1_ctrl_2, gpio_exp_1_pins) =
        i2c_io_expander::setup::<_, _, TCAL9539, 16>(i2c_bus);
    let mut gpio_exp_1_pins = Vec::from(gpio_exp_1_pins);
    let mut joy_up = gpio_exp_1_pins.remove(0);
    let mut joy_right = gpio_exp_1_pins.remove(0);
    let mut joy_down = gpio_exp_1_pins.remove(0);
    let mut joy_center = gpio_exp_1_pins.remove(0);
    let mut joy_left = gpio_exp_1_pins.remove(0);
    let mut button_a = gpio_exp_1_pins.remove(0);
    let mut button_b = gpio_exp_1_pins.remove(0);
    let disp_reset = gpio_exp_1_pins.remove(0);
    let sao_gpio_1 = gpio_exp_1_pins.remove(0);
    let tx_connect = gpio_exp_1_pins.remove(0);
    let tx_enable = gpio_exp_1_pins.remove(0);
    let mut _can_disconnect = gpio_exp_1_pins.remove(0);
    let mut _sd_cd = gpio_exp_1_pins.remove(0);
    let term_sel0 = gpio_exp_1_pins.remove(0);
    let term_sel1 = gpio_exp_1_pins.remove(0);
    let tx_rx_tie = gpio_exp_1_pins.remove(0);

    // Gpio Expander 2 (PCA9536)
    let (gpio_exp_2_ctrl, _, gpio_exp_2_pins) = i2c_io_expander::setup::<_, _, PCA9536, 4>(i2c_bus);
    let mut gpio_exp_2_pins = Vec::from(gpio_exp_2_pins);

    let pwr_injector = gpio_exp_2_pins.remove(0);
    let mut _pwr_trx = gpio_exp_2_pins.remove(0);
    let mut pwr_receiver = gpio_exp_2_pins.remove(0);
    let sao_gpio_2 = gpio_exp_2_pins.remove(0);

    // Battery charger (BQ25895)
    let (batt_ctrl_1, batt_ctrl_2, batt_runner) = bq25895::setup(i2c_bus);
    unwrap!(spawner.spawn(tasks::batt::batt_task(batt_ctrl_1)));

    // Peripheral interrupt handler.
    let int_runner = interrupt_i2c::setup(int_pin, gpio_exp_1_ctrl_1, batt_runner, accel_runner);
    unwrap!(spawner.spawn(tasks::irq::irq_task(int_runner)));

    // Set buttons as input.
    joy_up.set_direction(true).await;
    joy_right.set_direction(true).await;
    joy_down.set_direction(true).await;
    joy_center.set_direction(true).await;
    joy_left.set_direction(true).await;
    button_a.set_direction(true).await;
    button_b.set_direction(true).await;

    // Connect sd card to storage
    // storage.set_storage(sd_card);

    // USB Mass Storage Class
    // unwrap!(spawner.spawn(platform::msc::class::task(storage)));

    let (ctrl_channel, ctrl_ack) = make_ctrl_channel!();

    // Backlight test
    unwrap!(spawner.spawn(tasks::ctrl::ctrl_task(
        ctrl_channel.receiver(),
        ctrl_ack,
        p.PWM_SLICE0,
        p.PIN_16,
        disp_reset,
        term_sel0,
        term_sel1,
        tx_rx_tie,
        sao_gpio_1,
        sao_gpio_2,
    )));

    // NeoPixel App
    unwrap!(spawner.spawn(neopixel_task(
        unwrap!(app_watch.receiver()),
        app_ack_channel.sender(),
        led_channel.receiver(),
        ws2812,
        unwrap!(shake_watch.receiver()),
        shake_signal_ready,
        shake_signal_done
    )));

    // USB CLI.
    unwrap!(spawner.spawn(usb_cli::cli_task(
        cli,
        repl_in_channel.sender(),
        repl_out_channel.receiver()
    )));

    // Accelerometer (MC3479)
    accel_ctrl
        .set_mode(mc3479::registers::ModeState::Sleep, false, false)
        .await
        .unwrap();
    Timer::after_millis(100).await;
    accel_ctrl
        .set_mode(mc3479::registers::ModeState::Standby, false, false)
        .await
        .unwrap();
    Timer::after_millis(100).await;
    accel_ctrl
        .set_comm_control(true, false, false)
        .await
        .unwrap();
    Timer::after_millis(100).await;
    accel_ctrl
        .set_sample_rate(mc3479::registers::SampleRate::Idr50Hz)
        .await
        .unwrap();
    Timer::after_millis(100).await;
    accel_ctrl
        .set_motion_control(false, false, false, true, true, true, false, true)
        .await
        .unwrap();
    Timer::after_millis(100).await;
    accel_ctrl.set_anym_threshold(0x0fff).await.unwrap();
    Timer::after_millis(100).await;
    accel_ctrl.set_shake_threshold(0x0001).await.unwrap();
    Timer::after_millis(100).await;
    accel_ctrl.set_shake_duration(1, 1).await.unwrap();
    Timer::after_millis(100).await;
    accel_ctrl
        .set_interrupt_enable(false, false, false, false, false, false, false)
        .await
        .unwrap();
    Timer::after_millis(100).await;
    accel_ctrl
        .set_mode(mc3479::registers::ModeState::Wake, false, false)
        .await
        .unwrap();
    Timer::after_millis(100).await;

    warn!("BEGIN PIO");
    // Rx task
    let (rx_channel, rx_ack) = make_rx_channels!();
    unwrap!(spawner.spawn(tasks::rx::rx_task(
        rx_channel.receiver(),
        rx_ack,
        p.UART1,
        p.PIO2,
        p.PIN_9,
        p.DMA_CH4,
        pwr_receiver
    )));

    // Tx task
    let (tx_channel, tx_ack) = make_tx_channels!();
    unwrap!(spawner.spawn(tasks::tx::tx_task(
        tx_channel.receiver(),
        tx_ack,
        p.PIO1,
        p.DMA_CH5,
        p.PIN_18,
        p.PIN_19,
        p.PIN_20,
        p.PIN_21,
        p.PIN_22,
        p.PIN_23,
        p.PIN_24,
        p.PIN_25,
        p.PIN_26,
        p.PIN_27,
        p.PIN_28,
        tx_connect,
        tx_enable,
        pwr_injector,
    )));

    // RPC runtime
    debug!("Spawning RPC runtime!");
    let trng = Trng::new(p.TRNG, Irqs, embassy_rp::trng::Config::default());

    unwrap!(spawner.spawn(repl::rpc::repl_rpc(
        app_watch.sender(),
        app_ack_channel.receiver(),
        call_channel.receiver(),
        result_channel.sender(),
        display_channel.sender(),
        led_channel.sender(),
        tx_channel.sender(),
        tx_ack,
        rx_channel.sender(),
        rx_ack,
        ctrl_channel.sender(),
        ctrl_ack,
        accel_ctrl,
        batt_ctrl_2,
        gpio_exp_1_ctrl_2,
        gpio_exp_2_ctrl,
        trng,
    )));
}
