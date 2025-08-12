use super::data::Data;
use alloc::sync::Arc;
use defmt::info;
use embassy_embedded_hal::shared_bus::{asynch::i2c::I2cDevice, I2cDeviceError};
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_hal::i2c::ErrorType;
use embedded_hal_async::i2c::I2c;

use crate::platform::{
    mc3479::registers::{
        DecMode, LpfBw, ModeState, Range, Registers, SampleRate, TempPeriod, Tilt35,
    },
    util::bool_array_to_u8,
};

pub struct Control<M: RawMutex + 'static, D: I2c + ErrorType + 'static> {
    pub data: Arc<Mutex<M, Data>>,
    dev: I2cDevice<'static, M, D>,
}

impl<M: RawMutex + 'static, D: I2c + ErrorType + 'static> Control<M, D> {
    pub fn new(dev: I2cDevice<'static, M, D>, data: Arc<Mutex<M, Data>>) -> Self {
        Self { data, dev }
    }

    pub async fn read(&mut self) -> Data {
        // TODO: Code duplication from Runner.
        // let reg = [0x17];
        // let mut data = [0u8, 0u8];
        // self.dev.write_read(0x4c, &reg, &mut data).await.unwrap();
        // let chip_id = data[1];

        // error!("CHIP ID: {:x}", chip_id);

        // read device status
        let reg = [0x05];
        let mut data = [0u8];
        self.dev.write_read(0x4c, &reg, &mut data).await.unwrap();
        let device_status = data[0];

        // read data
        let reg = [0x0D];
        let mut data = [0u8; 6];
        self.dev.write_read(0x4c, &reg, &mut data).await.unwrap();

        // read status
        let reg = [0x13];
        let mut statuses = [0u8, 0u8];
        self.dev
            .write_read(0x4c, &reg, &mut statuses)
            .await
            .unwrap();

        // DONT clear interrupts. That happens in Control
        // let clear_int = [0x14u8, 0xffu8]; // send back interrupt status
        // self.dev.write(0x4c, &clear_int).await.unwrap();

        let mut data_guard = self.data.lock().await;
        data_guard.x = (data[1] as i16) << 8 | (data[0] as i16);
        data_guard.y = (data[3] as i16) << 8 | (data[2] as i16);
        data_guard.z = (data[5] as i16) << 8 | (data[4] as i16);
        data_guard.status = statuses[0].into();
        data_guard.interrupt_status = statuses[1].into();
        let ret = *data_guard;

        info!("Updated accel data from Control");
        info!(
            "{:x}: ({:?}, {:?}. {:?}) {:?} {:?}",
            device_status,
            data_guard.x,
            data_guard.y,
            data_guard.z,
            data_guard.status,
            data_guard.interrupt_status
        );
        ret
    }

    pub async fn set_register_8(
        &mut self,
        reg: u8,
        value: u8,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        let data = [reg, value];
        info!("SET_REG_8 {:x}: {:x}", reg, value);
        self.dev.write(0x4c, &data).await
    }

    pub async fn set_register_16(
        &mut self,
        reg: u8,
        value: u16,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        // send LSB first
        let data = [reg, value as u8, (value >> 8) as u8];
        self.dev.write(0x4c, &data).await
    }

    pub async fn set_interrupt_enable(
        &mut self,
        tilt: bool,
        flip: bool,
        anym: bool,
        shake: bool,
        tilt35: bool,
        auto_clr: bool,
        acq: bool,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        // let test = bool_array_to_u8(&[acq, auto_clr, false, tilt35, shake, anym, flip, tilt]);
        // info!("WOOT {:?}", test);
        self.set_register_8(
            Registers::IntrCtrl.into(),
            bool_array_to_u8(&[acq, auto_clr, false, tilt35, shake, anym, flip, tilt]),
        )
        .await
    }

    pub async fn set_mode(
        &mut self,
        mode: ModeState,
        i2c_wdt_neg: bool,
        i2c_wdt_pos: bool,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(
            Registers::Mode.into(),
            bool_array_to_u8(&[
                false,
                false,
                i2c_wdt_pos,
                i2c_wdt_neg,
                false,
                false,
                false,
                false,
            ]) | mode as u8,
        )
        .await
    }

    pub async fn set_sample_rate(
        &mut self,
        rate: SampleRate,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(Registers::SampleRate.into(), rate.into())
            .await
    }

    pub async fn set_motion_control(
        &mut self,
        reset: bool,
        raw_proc_stat: bool,
        z_axis_ort: bool,
        tilt35_en: bool,
        shake_en: bool,
        anym: bool,
        motion_latch: bool,
        tiltflip: bool,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(
            Registers::MotionCtrl.into(),
            bool_array_to_u8(&[
                reset,
                raw_proc_stat,
                z_axis_ort,
                tilt35_en,
                shake_en,
                anym,
                motion_latch,
                tiltflip,
            ]),
        )
        .await
    }

    pub async fn clear_interrupts(
        &mut self,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(Registers::IntrStat.into(), 0x00).await
    }

    pub async fn range_select(
        &mut self,
        range: Range,
        lpf_bw: LpfBw,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(Registers::Range.into(), (range as u8) << 4 | lpf_bw as u8)
            .await
    }

    // GAIN + offset
    pub async fn set_x_offset(
        &mut self,
        offset: i16,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_16(Registers::XOffL.into(), offset as u16)
            .await
    }

    pub async fn set_y_offset(
        &mut self,
        offset: i16,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_16(Registers::YOffL.into(), offset as u16)
            .await
    }

    pub async fn set_z_offset(
        &mut self,
        offset: i16,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_16(Registers::ZOffL.into(), offset as u16)
            .await
    }

    pub async fn set_fifo_control(
        &mut self,
        mode: bool,
        enable: bool,
        reset: bool,
        comb_int: bool,
        th_int: bool,
        full_int: bool,
        empty_int: bool,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(
            Registers::FifoCtrl.into(),
            bool_array_to_u8(&[
                false, mode, enable, reset, comb_int, th_int, full_int, empty_int,
            ]),
        )
        .await
    }

    pub async fn set_fifo_threshold(
        &mut self,
        threshold: u8,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(Registers::FifoTh.into(), threshold)
            .await
    }

    pub async fn set_fifo_control2(
        &mut self,
        burst: bool,
        wrap_addr: bool,
        wrap_en: bool,
        dec_mode: DecMode,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(
            Registers::FifoCtrl2.into(),
            bool_array_to_u8(&[burst, false, wrap_addr, wrap_en, false, false, false, false])
                | dec_mode as u8,
        )
        .await
    }

    pub async fn set_comm_control(
        &mut self,
        indiv_int_clr: bool,
        spi_3wire_en: bool,
        int1_int2_req_swap: bool,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(
            Registers::FifoCtrl2.into(),
            bool_array_to_u8(&[
                false,
                indiv_int_clr,
                spi_3wire_en,
                int1_int2_req_swap,
                false,
                false,
                false,
                false,
            ]),
        )
        .await
    }

    pub async fn set_gpio_control(
        &mut self,
        gpio2_intn2_ipp: bool,
        gpio2_intn2_iah: bool,
        gpio1_intn1_ipp: bool,
        gpio1_intn1_iah: bool,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(
            Registers::FifoCtrl2.into(),
            bool_array_to_u8(&[
                gpio2_intn2_ipp,
                gpio2_intn2_iah,
                false,
                false,
                gpio1_intn1_ipp,
                gpio1_intn1_iah,
                false,
                false,
            ]),
        )
        .await
    }

    pub async fn set_tilt_flip_threshold(
        &mut self,
        threshold: u16,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_16(Registers::TiltFlipThreshLsb.into(), threshold)
            .await
    }

    pub async fn set_tilt_flip_debounce(
        &mut self,
        debounce: u8,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(Registers::TiltFlipDebounce.into(), debounce)
            .await
    }

    pub async fn set_anym_threshold(
        &mut self,
        threshold: u16,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_16(Registers::AnyMotionThreshLsb.into(), threshold)
            .await
    }

    pub async fn set_anym_debounce(
        &mut self,
        debounce: u8,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(Registers::AnyMotionDebounce.into(), debounce)
            .await
    }

    pub async fn set_shake_threshold(
        &mut self,
        threshold: u16,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_16(Registers::ShakeThreshLsb.into(), threshold)
            .await
    }

    pub async fn set_shake_duration(
        &mut self,
        cnt: u8,
        p2p: u16,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_16(
            Registers::PeakToPeakDurationLsb.into(),
            (p2p << 8) | (((cnt as u16) & 0b111) << 4) | ((p2p & 0b0000111100000000) >> 8),
        )
        .await
    }

    pub async fn set_timer_control(
        &mut self,
        per_int_en: bool,
        period: TempPeriod,
        tilt35: Tilt35,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(
            Registers::TimerControl.into(),
            bool_array_to_u8(&[per_int_en, false, false, false, false, false, false, false])
                | (period as u8) << 4
                | tilt35 as u8,
        )
        .await
    }

    pub async fn set_read_count(
        &mut self,
        count: u8,
    ) -> Result<(), I2cDeviceError<<D as ErrorType>::Error>> {
        self.set_register_8(Registers::ReadCount.into(), count)
            .await
    }
}
