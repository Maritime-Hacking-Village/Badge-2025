use crate::{
    apps::{console::ConsoleDisplay, scrolling_console::ScrollingConsole},
    platform::{
        mc3479::runner::{ShakeReceiver, ShakeSignal},
        repl::{
            console::{ConsoleReader, CONSOLE_MTU},
            display::{DisplayCommand, DisplayReceiver},
            rpc::{AppAck, AppAckSender, AppContextReceiver},
        },
    },
};
use defmt::warn;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_futures::select::{self, Either, Either3};
use embassy_rp::{
    gpio::Output,
    peripherals::SPI0,
    spi::{self, Spi},
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Instant, Timer};
use embedded_graphics::{
    image::ImageDrawable, mono_font::ascii::FONT_7X13, pixelcolor::Rgb565, prelude::*,
    primitives::Rectangle,
};
use embedded_io_async::Write;
use mipidsi::{interface::SpiInterfaceAsync, models::ST7789, DisplayAsync, NoResetPin};

#[embassy_executor::task]
pub async fn display_task(
    mut app_rx: AppContextReceiver,
    app_ack_tx: AppAckSender,
    console_reader: ConsoleReader,
    display_recv: DisplayReceiver,
    mut display: DisplayAsync<
        'static,
        SpiInterfaceAsync<
            SpiDeviceWithConfig<
                'static,
                NoopRawMutex,
                Spi<'static, SPI0, spi::Async>,
                Output<'static>,
            >,
            Output<'static>,
        >,
        ST7789,
        NoResetPin,
    >,
    // mut display: DisplayAsync<
    //     'static,
    //     SpiInterfaceAsync<
    //         SpiDeviceWithConfig<
    //             'static,
    //             NoopRawMutex,
    //             Spi<'static, SPI0, spi::Async>,
    //             Output<'static>,
    //         >,
    //         Output<'static>,
    //     >,
    //     ST7789,
    //     Output<'static>,
    // >,
    mut shake_rx: ShakeReceiver,
    shake_signal_ready: &'static ShakeSignal,
    shake_signal_done: &'static ShakeSignal,
) -> ! {
    let puke = tinygif::Gif::<Rgb565>::from_slice(include_bytes!("../../assets/puke.gif")).unwrap();

    // Draw a square:
    // Rectangle::new(Point::new(0, 0), Size::new(320, 170))
    //     .into_styled(
    //         PrimitiveStyleBuilder::new()
    //             .stroke_width(1)
    //             .stroke_color(Rgb565::RED)
    //             .fill_color(Rgb565::CYAN)
    //             .build(),
    //     )
    //     .draw(&mut display.translated(Point::new(0, 35)))
    //     .unwrap();
    // display.flush().await.unwrap();

    let mut buf: [u8; CONSOLE_MTU] = [0_u8; CONSOLE_MTU];
    let _ = display.clear(Rgb565::RED);
    let _ = display.flush().await;
    let cropped_rectangle = Rectangle::new(Point::new(35, 0), Size::new(170, 320));
    let mut console = ScrollingConsole::new(display, FONT_7X13, cropped_rectangle).await;

    loop {
        'system: loop {
            match select::select3(
                console_reader.read(&mut buf),
                app_rx.changed(),
                shake_rx.changed(),
            )
            .await
            {
                Either3::First(count) => {
                    let _ = console.write(&buf[..count]).await;
                    let _ = console.flush().await;
                }
                Either3::Second(user_control) => {
                    if user_control {
                        warn!("Exiting display system application context!");
                        app_ack_tx.send(AppAck::Display).await;
                        break 'system;
                    }
                }
                Either3::Third(shooketh) => {
                    if shooketh {
                        warn!("Getting ready to blow chunks!");
                        shake_signal_ready.wait().await;
                        warn!("Got the chunky signal!");
                        let mut display = console.into_inner().await;
                        let mut console_display = ConsoleDisplay::new(display, FONT_7X13).await;
                        console_display.get_display().clear(Rgb565::BLACK).unwrap();
                        console_display.flush().await;
                        let start = Instant::now();

                        for frame in puke.frames() {
                            frame.draw(console_display.get_rotated()).unwrap();
                            let _ = console_display.flush().await;
                            let remain_delay = ((frame.delay_centis as u64) * 10)
                                .saturating_sub(start.elapsed().as_millis());
                            Timer::after_millis(remain_delay).await;
                        }

                        // Let the flag stay on the screen for a while.
                        warn!("Sending that we're done blowing chunks!");
                        shake_signal_done.signal(());
                        warn!("Done sending that we're done blowing chunks!");
                        console_display.get_display().clear(Rgb565::RED).unwrap();
                        console_display.flush().await;
                        display = console_display.into_inner();
                        console =
                            ScrollingConsole::new(display, FONT_7X13, cropped_rectangle).await;
                    }
                }
            }
        }

        let display = console.into_inner().await;
        let mut console_display = ConsoleDisplay::new(display, FONT_7X13).await;
        console_display.get_display().clear(Rgb565::BLACK).unwrap();
        console_display.flush().await;

        'user: loop {
            match select::select(display_recv.receive(), app_rx.changed()).await {
                Either::First(command) => match command {
                    DisplayCommand::ConsoleWrite(text) => {
                        warn!("ConsoleWrite");
                        let _ = console_display.write(text.as_bytes()).await;
                    }
                    DisplayCommand::SetPixel(x, y, color) => {
                        warn!("SetPixel");
                        let _ = console_display
                            .get_rotated()
                            .draw_iter([Pixel(Point::new(x as i32, y as i32), color)].into_iter());
                    }
                    DisplayCommand::FillRegion(sx, ex, sy, ey, color) => {
                        warn!("FillRegion");
                        let _ = console_display.get_rotated().fill_solid(
                            &Rectangle::new(
                                Point::new(sx as i32, sy as i32),
                                Size::new((ex - sx) as u32, (ey - sy) as u32),
                            ),
                            color,
                        );
                    }
                    DisplayCommand::Clear => {
                        warn!("UpdateFrame");
                        let _ = console_display.get_display().clear(Rgb565::BLACK);
                    }
                    DisplayCommand::Flush => {
                        warn!("Flushing display!");
                        // NOTE: Suppressing SPI errors for production.
                        console_display.flush().await;
                    }
                },
                Either::Second(user_control) => {
                    if !user_control {
                        warn!(
                            "Returning to display system application context! {}",
                            user_control
                        );
                        // Clear the console buffer before looping back.
                        let _ = console_reader.try_read(&mut buf);
                        let mut display = console_display.into_inner();
                        let _ = display.clear(Rgb565::RED);
                        let _ = display.flush().await;
                        console =
                            ScrollingConsole::new(display, FONT_7X13, cropped_rectangle).await;
                        // TODO: In the future, may want to have the app_watch be multi-state to support
                        //       multi-phase acknowledgement to keep all state transitions in lock-step.
                        app_ack_tx.send(AppAck::Display).await;
                        break 'user;
                    }
                }
            }
        }
    }
}
