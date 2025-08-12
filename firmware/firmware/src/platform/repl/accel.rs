use crate::{
    platform::repl::{
        rpc::{RpcCall, RpcCallSender, RpcResult, RpcResultReceiver},
        rpc_call,
    },
    register_repl_fn,
};
use alloc::boxed::Box;
use rhai::{Dynamic, Engine, EvalAltResult, Map, Module, NativeCallContext, INT};

pub(crate) fn repl_accel_read(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<Map, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelRead;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;
    let mut ret = Map::new();

    match result {
        RpcResult::AccelRead(data) => {
            ret.insert("x".into(), Dynamic::from_int(data.x as i64));
            ret.insert("y".into(), Dynamic::from_int(data.y as i64));
            ret.insert("z".into(), Dynamic::from_int(data.z as i64));

            let mut status = Map::new();
            status.insert("tilt".into(), data.status.tilt.into());
            status.insert("flip".into(), data.status.flip.into());
            status.insert("anym".into(), data.status.anym.into());
            status.insert("shake".into(), data.status.shake.into());
            status.insert("tilt35".into(), data.status.tilt35.into());
            status.insert("fifo".into(), data.status.fifo.into());
            status.insert("new_data".into(), data.status.new_data.into());
            ret.insert("status".into(), status.into());

            let mut interrupt_status = Map::new();
            interrupt_status.insert("tilt_int".into(), data.interrupt_status.tilt_int.into());
            interrupt_status.insert("flip_int".into(), data.interrupt_status.flip_int.into());
            interrupt_status.insert("anym_int".into(), data.interrupt_status.anym_int.into());
            interrupt_status.insert("shake_int".into(), data.interrupt_status.shake_int.into());
            interrupt_status.insert("tilt35_int".into(), data.interrupt_status.tilt35_int.into());
            interrupt_status.insert("acq_int".into(), data.interrupt_status.acq_int.into());
            ret.insert("interrupt_status".into(), interrupt_status.into());
        }
        _ => {
            unreachable!()
        }
    }

    Ok(ret)
}

pub(crate) fn repl_accel_set_reg8(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    reg: INT,
    value: INT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelSetReg8(reg as u8, value as u8);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_accel_set_reg16(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    reg: INT,
    value: INT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelSetReg16(reg as u8, value as u16);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_accel_set_int_enable(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    tilt: bool,
    flip: bool,
    anym: bool,
    shake: bool,
    tilt35: bool,
    auto_clr: bool,
    acq: bool,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelSetIntEnable(tilt, flip, anym, shake, tilt35, auto_clr, acq);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_accel_set_mode(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    mode: INT,
    i2c_wdt_neg: bool,
    i2c_wdt_pos: bool,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelSetMode((mode as u8).into(), i2c_wdt_neg, i2c_wdt_pos);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_accel_set_sample_rate(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    rate: INT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelSetSampleRate((rate as u8).into());
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_accel_set_motion_control(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    reset: bool,
    raw_proc_stat: bool,
    z_axis_ort: bool,
    tilt35_en: bool,
    shake_en: bool,
    anym: bool,
    motion_latch: bool,
    tiltflip: bool,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelSetMotionControl(
        reset,
        raw_proc_stat,
        z_axis_ort,
        tilt35_en,
        shake_en,
        anym,
        motion_latch,
        tiltflip,
    );
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

// pub(crate) fn repl_accel_clear_interrupts(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelClearInterrupts;
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_range_select(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     range: INT,
//     lpf_bw: INT,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelRangeSelect((range as u8).into(), (lpf_bw as u8).into());
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_x_offset(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     offset: INT,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetXOffset(offset as i16);
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_y_offset(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     offset: INT,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetYOffset(offset as i16);
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_z_offset(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     offset: INT,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetZOffset(offset as i16);
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_fifo_control(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     mode: bool,
//     enable: bool,
//     reset: bool,
//     comb_int: bool,
//     th_int: bool,
//     full_int: bool,
//     empty_int: bool,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call =
//         RpcCall::AccelSetFifoControl(mode, enable, reset, comb_int, th_int, full_int, empty_int);
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_fifo_threshold(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     threshold: INT,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetFifoThreshold(threshold as u8);
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_fifo_control2(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     burst: bool,
//     wrap_addr: bool,
//     wrap_en: bool,
//     dec_mode: INT,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetFifoControl2(burst, wrap_addr, wrap_en, (dec_mode as u8).into());
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_comm_control(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     indiv_int_clr: bool,
//     spi_3wire_en: bool,
//     int1_int2_req_swap: bool,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetCommControl(indiv_int_clr, spi_3wire_en, int1_int2_req_swap);
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_gpio_control(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     gpio2_intn2_ipp: bool,
//     gpio2_intn2_iah: bool,
//     gpio1_intn1_ipp: bool,
//     gpio1_intn1_iah: bool,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetGpioControl(
//         gpio2_intn2_ipp,
//         gpio2_intn2_iah,
//         gpio1_intn1_ipp,
//         gpio1_intn1_iah,
//     );
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_tilt_flip_threshold(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     threshold: INT,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetTiltFlipThreshold(threshold as u16);
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_tilt_flip_debounce(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     debounce: INT,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetTiltFlipDebounce(debounce as u8);
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

pub(crate) fn repl_accel_set_anym_threshold(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    threshold: INT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelSetAnymThreshold(threshold as u16);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_accel_set_anym_debounce(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    debounce: INT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelSetAnymDebounce(debounce as u8);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_accel_set_shake_threshold(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    threshold: INT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelSetShakeThreshold(threshold as u16);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_accel_set_shake_duration(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    cnt: INT,
    p2p: INT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::AccelSetShakeDuration(cnt as u8, p2p as u16);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

// pub(crate) fn repl_accel_set_timer_control(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     per_int_en: bool,
//     period: INT,
//     tilt35: FLOAT,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetTimerControl(per_int_en, (period as u16).into(), tilt35.into());
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

// pub(crate) fn repl_accel_set_read_count(
//     ctx: &NativeCallContext,
//     call_tx: RpcCallSender,
//     result_rx: RpcResultReceiver,
//     count: INT,
// ) -> Result<(), Box<EvalAltResult>> {
//     // Construct the RpcCall and send it non-blocking (errors if unable to send).
//     let call = RpcCall::AccelSetReadCount(count as u8);
//     let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

//     Ok(())
// }

pub(crate) fn register_functions(
    engine: &mut Engine,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) {
    let mut module = Module::new();
    register_repl_fn!(module, call_tx, result_rx, repl_accel_read, "read", ());
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_accel_set_reg8,
        "set_reg8",
        (reg: INT, value: INT)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_accel_set_reg16,
        "set_reg16",
        (reg: INT, value: INT)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_accel_set_int_enable,
        "set_int_enable",
        (tilt: bool, flip: bool, anym: bool, shake: bool, tilt35: bool, auto_clr: bool, acq: bool)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_accel_set_mode,
        "set_mode",
        (mode: INT, i2c_wdt_neg: bool, i2c_wdt_pos: bool)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_accel_set_sample_rate,
        "set_sample_rate",
        (rate: INT)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_accel_set_motion_control,
        "set_motion_control",
        (reset: bool, raw_proc_stat: bool, z_axis_ort: bool, tilt35_en: bool, shake_en: bool, anym: bool, motion_latch: bool, tiltflip: bool)
    );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_clear_interrupts,
    //     "clear_interrupts",
    //     ()
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_range_select,
    //     "range_select",
    //     (range: INT, lpf_bw: INT)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_x_offset,
    //     "set_x_offset",
    //     (offset: INT)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_y_offset,
    //     "set_y_offset",
    //     (offset: INT)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_z_offset,
    //     "set_z_offset",
    //     (offset: INT)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_fifo_control,
    //     "set_fifo_control",
    //     (mode: bool, enable: bool, reset: bool, comb_int: bool, th_int: bool, full_int: bool, empty_int: bool)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_fifo_threshold,
    //     "set_fifo_threshold",
    //     (threshold: INT)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_fifo_control2,
    //     "set_fifo_control2",
    //     (burst: bool, wrap_addr: bool, wrap_en: bool, dec_mode: INT)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_comm_control,
    //     "set_comm_control",
    //     (indiv_int_clr: bool, spi_3wire_en: bool, int1_int2_req_swap: bool)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_gpio_control,
    //     "set_gpio_control",
    //     (gpio2_intn2_ipp: bool, gpio2_intn2_iah: bool, gpio1_intn1_ipp: bool, gpio1_intn1_iah: bool)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_tilt_flip_threshold,
    //     "set_tilt_flip_threshold",
    //     (threshold: INT)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_tilt_flip_debounce,
    //     "set_tilt_flip_debounce",
    //     (debounce: INT)
    // );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_accel_set_anym_threshold,
        "set_anym_threshold",
        (threshold: INT)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_accel_set_anym_debounce,
        "set_anym_debounce",
        (debounce: INT)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_accel_set_shake_threshold,
        "set_shake_threshold",
        (debounce: INT)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_accel_set_shake_duration,
        "set_shake_duration",
        (cnt: INT, p2p: INT)
    );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_timer_control,
    //     "set_timer_control",
    //     (per_int_en: bool, period: INT, tilt35: FLOAT)
    // );
    // register_repl_fn!(
    //     module,
    //     call_tx,
    //     result_rx,
    //     repl_accel_set_read_count,
    //     "set_read_count",
    //     (count: INT)
    // );
    engine.register_static_module("accel", module.into());
}
