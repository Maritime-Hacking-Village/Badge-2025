use crate::platform::bq25895;
use defmt::warn;
use embassy_rp::{i2c, peripherals::I2C0};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Timer;

#[embassy_executor::task]
pub async fn batt_task(
    mut control: bq25895::control::Control<
        CriticalSectionRawMutex,
        i2c::I2c<'static, I2C0, i2c::Async>,
    >,
) -> ! {
    Timer::after_secs(1).await;

    let mut offset = bq25895::registers::reg06::Reg06::default();
    offset.vrechg = true;
    control.set_register(offset).await.unwrap();

    let mut enable_adc = bq25895::registers::reg02::Reg02::default();
    enable_adc.conv_rate = true;
    control.set_register(enable_adc).await.unwrap();
    // let value = control
    //     .get_register::<bq25895::registers::reg0e::Reg0e>()
    //     .await
    //     .unwrap();
    // warn!("Got battery voltage! {}", value);

    loop {
        let mut reg03 = bq25895::registers::reg03::Reg03::default();
        reg03.bat_loaden = false;
        reg03.wd_rst = true;
        reg03.sys_min = 3500u32.into();
        // reg03.chg_config = true;
        // control.set_register(reg03).await.unwrap();

        let reg12 = control
            .get_register::<bq25895::registers::reg12::Reg12>()
            .await
            .unwrap();
        warn!("Got REG12! {}", reg12);
        let reg02 = control
            .get_register::<bq25895::registers::reg02::Reg02>()
            .await
            .unwrap();
        warn!("Got REG02! {}", reg02);
        let reg0b = control
            .get_register::<bq25895::registers::reg0b::Reg0b>()
            .await
            .unwrap();
        warn!("Got REG0B! {}", reg0b);
        let reg03 = control
            .get_register::<bq25895::registers::reg03::Reg03>()
            .await
            .unwrap();
        warn!("Got REG03! {}", reg03);
        let reg0c = control
            .get_register::<bq25895::registers::reg0c::Reg0c>()
            .await
            .unwrap();
        warn!("Got REG0C! {}", reg0c);
        let mut reg07 = control
            .get_register::<bq25895::registers::reg07::Reg07>()
            .await
            .unwrap();
        warn!("Got REG07! {}", reg07);

        let mut set_reg07 = true;

        if reg0c.bat_fault {
            // Disable LED if the battery is disconnected.
            if !reg07.stat_dis {
                reg07.stat_dis = true;
                set_reg07 = true;
            }
        } else if reg07.stat_dis {
            reg07.stat_dis = false;
            set_reg07 = true;
        }

        if set_reg07 {
            control.set_register(reg07).await.unwrap();
        }

        Timer::after_secs(100).await;
    }
}
