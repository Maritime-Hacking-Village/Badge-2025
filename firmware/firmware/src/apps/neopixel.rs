use crate::platform::{
    mc3479::runner::{ShakeReceiver, ShakeSignal},
    neopixel::NUM_LEDS,
    repl::{
        led::LedReceiver,
        rpc::{AppAck, AppAckSender, AppContextReceiver},
    },
};
use defmt::warn;
use embassy_futures::select::{self, Either, Either3};
use embassy_rp::{peripherals::PIO0, pio_programs::ws2812::PioWs2812};
use embassy_time::{Duration, Ticker};
use smart_leds::RGB8;

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

fn mhv_pattern(iteration: usize) -> Option<bool> {
    match iteration {
        // M - -
        0 | 1 | 2 => Some(true),
        3 => Some(false),
        4 | 5 | 6 => Some(true),
        // IFS
        7 | 8 | 9 => Some(false),
        // H . . . .
        10 => Some(true),
        11 => Some(false),
        12 => Some(true),
        13 => Some(false),
        14 => Some(true),
        15 => Some(false),
        16 => Some(true),
        17 => Some(false),
        // IFS
        18 | 19 | 20 => Some(false),
        // V . . . -
        21 => Some(true),
        22 => Some(false),
        23 => Some(true),
        24 => Some(false),
        25 => Some(true),
        26 => Some(false),
        27 | 28 | 29 => Some(true),
        // IFS
        30 | 31 | 32 => Some(false),
        _ => None,
    }
}

fn sos_pattern(iteration: usize) -> Option<bool> {
    match iteration {
        // S
        0 => Some(true),
        1 => Some(false),
        2 => Some(true),
        3 => Some(false),
        4 => Some(true),
        // IFS
        5 | 6 | 7 => Some(false),
        // O
        8 | 9 | 10 => Some(true),
        11 => Some(false),
        12 | 13 | 14 => Some(true),
        15 => Some(false),
        16 | 17 | 18 => Some(true),
        // IFS
        19 | 20 | 21 => Some(false),
        // S
        22 => Some(true),
        23 => Some(false),
        24 => Some(true),
        25 => Some(false),
        26 => Some(true),
        // IFS
        27 | 28 | 29 => Some(false),
        // O
        30 | 31 | 32 => Some(true),
        33 => Some(false),
        34 | 35 | 36 => Some(true),
        37 => Some(false),
        38 | 39 | 40 => Some(true),
        // IFS
        41 | 42 | 43 => Some(false),
        _ => None,
    }
}

async fn weird_pattern(
    leds: &mut [RGB8; NUM_LEDS],
    ws2812: &mut PioWs2812<'static, PIO0, 0, NUM_LEDS>,
) {
    let grn = RGB8::new(0x00, 0xFF, 0x00) / 2;
    let nil = RGB8::new(0x00, 0x00, 0x00);
    let mhv = RGB8::new(0x48, 0x4D, 0x56);
    let c0 = RGB8::new(100, 13, 95);
    let c1 = RGB8::new(200, 70, 0);
    let c2 = RGB8::new(20, 70, 120);
    let mut ticker = Ticker::every(Duration::from_millis(200));
    let mut tick = 0_usize;
    let mut morse_tick = 0_usize;
    let mut morse_cnt = 1_usize;
    let morse_spacing = 1;

    loop {
        match mhv_pattern(morse_tick) {
            Some(mhv_on) => {
                if mhv_on {
                    leds[8] = mhv;
                } else {
                    leds[8] = nil;
                }

                if morse_cnt % morse_spacing == 0 {
                    morse_tick += 1;
                }
            }
            None => {
                morse_tick = 0;
                leds[8] = mhv;
            }
        }

        morse_cnt += 1;

        // LED pattern
        // 8  7  0  1  2
        //    6 5 4 3

        match tick {
            0 => {
                leds[7] = c0;
                leds[3] = c0;
                leds[6] = c1;
                leds[1] = c1;
                leds[0] = mhv;
                leds[4] = c2;
                leds[5] = c2;
            }
            1 => {
                leds[7] = c0;
                leds[3] = c0;
                leds[6] = c1;
                leds[1] = c1;
                leds[0] = c2;
                leds[4] = mhv;
                leds[5] = c2;
            }
            2 => {
                leds[7] = c1;
                leds[3] = c1;
                leds[6] = c0;
                leds[1] = c0;
                leds[0] = c2;
                leds[4] = c2;
                leds[5] = mhv;
            }
            3 => {
                leds[7] = c1;
                leds[3] = c1;
                leds[6] = c0;
                leds[1] = c0;
                leds[0] = mhv;
                leds[4] = c2;
                leds[5] = c2;
            }
            4 => {
                leds[7] = c0;
                leds[3] = c0;
                leds[6] = c1;
                leds[1] = c1;
                leds[0] = c2;
                leds[4] = mhv;
                leds[5] = c2;
            }
            5 => {
                leds[7] = c0;
                leds[3] = c0;
                leds[6] = c1;
                leds[1] = c1;
                leds[0] = c2;
                leds[4] = c2;
                leds[5] = mhv;
            }
            6 => {
                leds[7] = c1;
                leds[3] = c1;
                leds[6] = c0;
                leds[1] = c0;
                leds[0] = mhv;
                leds[4] = c2;
                leds[5] = c2;
            }
            _ => {
                leds[7] = c1;
                leds[3] = c1;
                leds[6] = c0;
                leds[1] = c0;
                leds[0] = c2;
                leds[4] = mhv;
                leds[5] = c2;
                tick = 0;
            }
        }

        leds[2] = grn;
        tick += 1;
        ws2812.write(leds).await;
        ticker.next().await;
    }
}

async fn rainbow_pattern(
    leds: &mut [RGB8; NUM_LEDS],
    ws2812: &mut PioWs2812<'static, PIO0, 0, NUM_LEDS>,
) {
    let grn = RGB8::new(0x00, 0xFF, 0x00) / 2;
    let nil = RGB8::new(0x00, 0x00, 0x00);
    let mhv = RGB8::new(0x48, 0x4D, 0x56);
    let mut ticker = Ticker::every(Duration::from_millis(10));
    let mut morse_tick = 0_usize;
    let mut morse_cnt = 1_usize;
    let mut sos_cnt = 1_usize;
    let mut sos_tick = 0_usize;
    let morse_spacing = 20;

    loop {
        for j in 0..(256 * 5) {
            for mut i in 0..8 {
                if i == 2 {
                    i += 1;
                }

                leds[i] = wheel(
                    ((((if i > 2 { i - 1 } else { i }) * 256) as u16 / 7 as u16 + j as u16) & 255)
                        as u8,
                );
                // Dim the color a little for battery saving and blue theme.
                leds[i] /= 4;
            }

            match mhv_pattern(morse_tick) {
                Some(mhv_on) => {
                    if mhv_on {
                        leds[8] = mhv;
                    } else {
                        leds[8] = nil;
                    }

                    if morse_cnt % morse_spacing == 0 {
                        morse_tick += 1;
                    }
                }
                None => {
                    morse_tick = 0;
                    leds[8] = mhv;
                }
            }

            match sos_pattern(sos_tick) {
                Some(mhv_on) => {
                    if mhv_on {
                        leds[2] = grn;
                    } else {
                        leds[2] = nil;
                    }

                    if sos_cnt % morse_spacing == 0 {
                        sos_tick += 1;
                    }
                }
                None => {
                    sos_tick = 0;
                    leds[2] = grn;
                }
            }

            morse_cnt += 1;
            sos_cnt += 1;
            ws2812.write(leds).await;
            ticker.next().await;
        }
    }
}

async fn sponsor_pattern(
    leds: &mut [RGB8; NUM_LEDS],
    ws2812: &mut PioWs2812<'static, PIO0, 0, NUM_LEDS>,
) {
    let grn = RGB8::new(0x00, 0xFF, 0x00) / 2;
    let nil = RGB8::new(0x00, 0x00, 0x00);
    let mhv = RGB8::new(0x48, 0x4D, 0x56);
    let mut ticker = Ticker::every(Duration::from_millis(10));
    let mut morse_tick = 0_usize;
    let mut morse_cnt = 1_usize;
    let morse_spacing = 20;

    loop {
        for j in 0..(256 * 5) {
            for mut i in 0..8 {
                if i == 2 {
                    i += 1;
                }

                leds[i] = wheel(
                    ((((if i > 2 { i - 1 } else { i }) * 256) as u16 / 7 as u16 + j as u16) & 255)
                        as u8,
                );
                // Dim the color a little for battery saving and blue theme.
                leds[i].b += leds[i].g / 2;
                leds[i].g /= 4;
                leds[i].g += leds[i].r / 4;
                leds[i] /= 4;
            }

            match mhv_pattern(morse_tick) {
                Some(mhv_on) => {
                    if mhv_on {
                        leds[8] = mhv;
                    } else {
                        leds[8] = nil;
                    }

                    if morse_cnt % morse_spacing == 0 {
                        morse_tick += 1;
                    }
                }
                None => {
                    morse_tick = 0;
                    leds[8] = mhv;
                }
            }

            morse_cnt += 1;
            leds[2] = grn;
            ws2812.write(leds).await;
            ticker.next().await;
        }
    }
}

async fn human_pattern(
    leds: &mut [RGB8; NUM_LEDS],
    ws2812: &mut PioWs2812<'static, PIO0, 0, NUM_LEDS>,
) {
    let grn = RGB8::new(0x00, 0xFF, 0x00) / 2;
    let nil = RGB8::new(0x00, 0x00, 0x00);
    let mhv = RGB8::new(0x48, 0x4D, 0x56);
    let mut ticker = Ticker::every(Duration::from_millis(10));
    let mut morse_tick = 0_usize;
    let mut morse_cnt = 1_usize;
    let morse_spacing = 20;

    loop {
        for j in 0..(256 * 5) {
            for mut i in 0..8 {
                if i == 2 {
                    i += 1;
                }

                leds[i] = wheel(
                    ((((if i > 2 { i - 1 } else { i }) * 256) as u16 / 7 as u16 + j as u16) & 255)
                        as u8,
                );
                // Dim the color a little for battery saving and blue theme.
                leds[i].b += leds[i].g / 2;
                leds[i].g /= 4;
                leds[i].g += leds[i].r / 4;
                leds[i].r /= 8;
                leds[i] /= 3;
            }

            match mhv_pattern(morse_tick) {
                Some(mhv_on) => {
                    if mhv_on {
                        leds[8] = mhv;
                    } else {
                        leds[8] = nil;
                    }

                    if morse_cnt % morse_spacing == 0 {
                        morse_tick += 1;
                    }
                }
                None => {
                    morse_tick = 0;
                    leds[8] = mhv;
                }
            }

            morse_cnt += 1;
            leds[2] = grn;
            ws2812.write(leds).await;
            ticker.next().await;
        }
    }
}

async fn teal_pattern(
    leds: &mut [RGB8; NUM_LEDS],
    ws2812: &mut PioWs2812<'static, PIO0, 0, NUM_LEDS>,
) {
    let grn = RGB8::new(0x00, 0xFF, 0x00) / 2;
    let nil = RGB8::new(0x00, 0x00, 0x00);
    let mhv = RGB8::new(0x48, 0x4D, 0x56);
    let mut ticker = Ticker::every(Duration::from_millis(10));
    let mut morse_tick = 0_usize;
    let mut morse_cnt = 1_usize;
    let morse_spacing = 20;

    loop {
        for j in 0..(256 * 5) {
            for mut i in 0..8 {
                if i == 2 {
                    i += 1;
                }

                leds[i] = wheel(
                    ((((if i > 2 { i - 1 } else { i }) * 256) as u16 / 7 as u16 + j as u16) & 255)
                        as u8,
                );
                // Dim the color a little for battery saving and blue theme.
                leds[i].b += leds[i].g / 2;
                leds[i].g /= 4;
                leds[i].g += leds[i].r / 4;
                leds[i].r /= 8;
                leds[i] /= 3;

                leds[i] /= 4;
            }

            match mhv_pattern(morse_tick) {
                Some(mhv_on) => {
                    if mhv_on {
                        leds[8] = mhv;
                    } else {
                        leds[8] = nil;
                    }

                    if morse_cnt % morse_spacing == 0 {
                        morse_tick += 1;
                    }
                }
                None => {
                    morse_tick = 0;
                    leds[8] = mhv;
                }
            }

            morse_cnt += 1;
            leds[2] = grn;
            ws2812.write(leds).await;
            ticker.next().await;
        }
    }
}

#[embassy_executor::task]
pub async fn neopixel_task(
    mut app_rx: AppContextReceiver,
    app_ack_tx: AppAckSender,
    led_recv: LedReceiver,
    mut ws2812: PioWs2812<'static, PIO0, 0, NUM_LEDS>,
    mut shake_rx: ShakeReceiver,
    shake_signal_ready: &'static ShakeSignal,
    shake_signal_done: &'static ShakeSignal,
) -> ! {
    loop {
        let mut leds = [RGB8::default(); NUM_LEDS];
        // GRB ordering for NeoPixel data line.
        leds[9] = RGB8::new(0x4C, 0x7B, 0x45);
        leds[10] = RGB8::new(0x50, 0x44, 0x6F);
        leds[11] = RGB8::new(0x73, 0x69, 0x6F);
        leds[12] = RGB8::new(0x69, 0x6E, 0x6E);
        leds[13] = RGB8::new(0x21, 0x67, 0x7D);
        ws2812.write(&leds).await;

        'system: loop {
            match select::select3(
                rainbow_pattern(&mut leds, &mut ws2812),
                app_rx.changed(),
                shake_rx.changed(),
            )
            .await
            {
                Either3::First(_) => {
                    // unreachable!()
                }
                Either3::Second(user_control) => {
                    if user_control {
                        warn!("Exiting NeoPixel system application context!");
                        app_ack_tx.send(AppAck::NeoPixel).await;
                        break 'system;
                    }
                }
                Either3::Third(shooketh) => {
                    if shooketh {
                        warn!("Setting LEDs to blow chunks!");

                        for j in 0..9 {
                            leds[j] = RGB8::new(0xA4, 0xB0, 0x05);
                        }

                        // Multiple writes in case the PIO sucks.
                        for _ in 0..10 {
                            ws2812.write(&leds).await;
                        }

                        warn!("Signalling that we're ready to blow chunks!");
                        shake_signal_ready.signal(());
                        warn!("Waiting for the puke animation to be over!");
                        shake_signal_done.wait().await;
                        warn!("Got the signal the puke animation is over!");
                    }
                }
            }
        }

        leds.fill(RGB8::default());
        leds[9] = RGB8::new(0x4C, 0x7B, 0x45);
        leds[10] = RGB8::new(0x50, 0x44, 0x6F);
        leds[11] = RGB8::new(0x73, 0x69, 0x6F);
        leds[12] = RGB8::new(0x69, 0x6E, 0x6E);
        leds[13] = RGB8::new(0x21, 0x67, 0x7D);
        ws2812.write(&leds).await;

        'user: loop {
            match select::select(led_recv.receive(), app_rx.changed()).await {
                Either::First((i, led)) => {
                    leds[i] = led;
                    ws2812.write(&leds).await;
                }
                Either::Second(user_control) => {
                    if !user_control {
                        warn!(
                            "Returning to NeoPixel system application context! {}",
                            user_control
                        );
                        app_ack_tx.send(AppAck::NeoPixel).await;
                        break 'user;
                    }
                }
            }
        }
    }
}
