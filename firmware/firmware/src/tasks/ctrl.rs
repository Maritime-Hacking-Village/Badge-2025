use core::cmp::min;

use crate::platform::{
    i2c_io_expander::{
        self,
        models::{pca9536::PCA9536, tcal9539::TCAL9539},
    },
    repl::{
        common::{AckSignal, ControlCommand, ControlReceiver},
        rpc::RpcResult,
    },
};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_rp::{
    i2c,
    peripherals::{I2C0, PIN_16, PWM_SLICE0},
    pwm::{Config, Pwm, SetDutyCycle},
    Peri,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Timer;
use embedded_hal_async::digital::{InputPin, StatefulOutputPin};

// TODO Getters for TERM and TIE.
/// This task handles all system pin control that is not state-dependent. That is, not bound to a mode of operation.
/// Display backlight (PWM)
/// Display reset (IO expander)
/// TERM_SEL0, TERM_SEL1
/// Tx-Rx tie
/// SAO GPIO 1/2
#[embassy_executor::task]
pub async fn ctrl_task(
    ctrl_rx: ControlReceiver,
    ctrl_ack: &'static AckSignal,
    pwm_slice: Peri<'static, PWM_SLICE0>,
    disp_backlight: Peri<'static, PIN_16>,
    mut disp_reset: i2c_io_expander::pin::Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        TCAL9539,
    >,
    mut term_sel0: i2c_io_expander::pin::Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        TCAL9539,
    >,
    mut term_sel1: i2c_io_expander::pin::Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        TCAL9539,
    >,
    mut tx_rx_tie: i2c_io_expander::pin::Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        TCAL9539,
    >,
    mut sao_gpio_1: i2c_io_expander::pin::Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        TCAL9539,
    >,
    mut sao_gpio_2: i2c_io_expander::pin::Pin<
        CriticalSectionRawMutex,
        I2cDevice<'static, CriticalSectionRawMutex, i2c::I2c<'static, I2C0, i2c::Async>>,
        PCA9536,
    >,
) {
    // Set default configuration for provided pins.
    // Tie enabled by default.
    tx_rx_tie.set_direction(false).await;
    tx_rx_tie.set_output(true).await;

    // TS5A3359DCUR logic:
    //
    // NO2:  13R
    // NO1: 220R
    // NO0: 120R
    // IN1: SEL_0
    // IN2: SEL_1
    //
    // IN2, IN1, COM
    //   0,   0, OFF
    //   0,   1, NO0
    //   1,   0, NO1
    //   1,   1, NO2
    //
    // SEL_1, SEL_0,   R
    //     0,     0, NaN
    //     0,     1, 120
    //     1,     0, 220
    //     1,     1,  13
    //
    // 120R default termination resistor.
    term_sel0.set_direction(false).await;
    term_sel1.set_direction(false).await;
    term_sel0.set_output(true).await;
    term_sel1.set_output(false).await;

    // Reset is active low.
    disp_reset.set_direction(false).await;
    disp_reset.set_output(true).await;

    // Defaults to input.
    sao_gpio_1.set_direction(false).await;
    sao_gpio_2.set_direction(false).await;

    // If we aim for a specific frequency, here is how we can calculate the top value.
    // The top value sets the period of the PWM cycle, so a counter goes from 0 to top and then wraps around to 0.
    // Every such wraparound is one PWM cycle. So here is how we get 25KHz:
    let desired_freq_hz = 25_000;
    let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
    let divider = 16u8;
    let period = (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1;
    let mut c = Config::default();
    c.top = period;
    c.divider = divider.into();

    let mut pwm = Pwm::new_output_a(pwm_slice, disp_backlight, c.clone());
    pwm.set_duty_cycle_percent(50).unwrap();

    loop {
        match ctrl_rx.receive().await {
            ControlCommand::SetDisplayBacklight(percent) => {
                let _ = pwm.set_duty_cycle_percent(min(100, percent));
                ctrl_ack.signal(Ok(RpcResult::DisplaySetBacklight));
            }
            ControlCommand::ResetDisplay => {
                disp_reset.set_output(false).await;
                Timer::after_micros(10).await;
                disp_reset.set_output(true).await;
                ctrl_ack.signal(Ok(RpcResult::DisplayReset));
            }
            ControlCommand::SetTerm(term_0, term_1) => {
                term_sel0.set_output(term_0).await;
                term_sel1.set_output(term_1).await;
                ctrl_ack.signal(Ok(RpcResult::TrxSetTerm));
            }
            ControlCommand::SetTxRxTie(enabled) => {
                tx_rx_tie.set_output(enabled).await;
                ctrl_ack.signal(Ok(RpcResult::TrxSetTxRxTie));
            }
            ControlCommand::GetSaoGpioDir => {
                let dir_1 = sao_gpio_1.get_direction().await;
                let dir_2 = sao_gpio_2.get_direction().await;
                ctrl_ack.signal(Ok(RpcResult::SaoGetDirection(dir_1, dir_2)))
            }
            ControlCommand::SetSaoGpioDir(dir_1, dir_2) => {
                sao_gpio_1.set_direction(dir_1).await;
                sao_gpio_2.set_direction(dir_2).await;
                ctrl_ack.signal(Ok(RpcResult::SaoSetDirection))
            }
            ControlCommand::WriteSaoGpio(output_1, output_2) => {
                sao_gpio_1.set_output(output_1).await;
                sao_gpio_2.set_output(output_2).await;
                ctrl_ack.signal(Ok(RpcResult::SaoWrite))
            }
            ControlCommand::ReadSaoGpio => {
                let value_1 = if sao_gpio_1.get_direction().await {
                    // Input
                    sao_gpio_1.is_high().await.unwrap_or(false)
                } else {
                    // Output
                    sao_gpio_1.is_set_high().await.unwrap_or(false)
                };
                let value_2 = if sao_gpio_2.get_direction().await {
                    // Input
                    sao_gpio_2.is_high().await.unwrap_or(false)
                } else {
                    // Output
                    sao_gpio_2.is_set_high().await.unwrap_or(false)
                };

                ctrl_ack.signal(Ok(RpcResult::SaoRead(value_1, value_2)))
            }
        }
    }
}
