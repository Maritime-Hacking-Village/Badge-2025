//! Differential receiver RPC calls

use crate::{
    apps::rx::RxMode,
    platform::repl::{
        rpc::{RpcCall, RpcCallSender, RpcResult, RpcResultReceiver},
        rpc_call,
    },
    register_repl_fn,
};
use alloc::{borrow::ToOwned, boxed::Box, string::String};
use bitvec::{order::Msb0, vec::BitVec};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel, signal};
use embassy_time::Duration;
use rhai::{Blob, Engine, EvalAltResult, ImmutableString, Module, NativeCallContext, FLOAT, INT};

// TODO: Move all channels to common.rs
#[derive(Debug, Clone)]
pub enum RxCommand {
    IsEnabled,
    EnableDisable(bool),
    GetBaud,
    SetBaud(u32),
    SetMode(RxMode),
    GetMode,
}

pub const RX_MTU: usize = 1;

pub type RxChannel = channel::Channel<CriticalSectionRawMutex, RxCommand, RX_MTU>;
pub type RxSender = channel::Sender<'static, CriticalSectionRawMutex, RxCommand, RX_MTU>;
pub type RxReceiver = channel::Receiver<'static, CriticalSectionRawMutex, RxCommand, RX_MTU>;

pub type RxBuffer = signal::Signal<CriticalSectionRawMutex, BitVec<u8, Msb0>>;

#[macro_export]
macro_rules! make_rx_channels {
    () => {{
        use crate::platform::repl::{
            common::AckSignal,
            rx::{RxBuffer, RxChannel},
        };
        use embassy_sync::lazy_lock::LazyLock;

        static CHANNEL: LazyLock<RxChannel> = LazyLock::new(|| RxChannel::new());
        static SIGNAL: LazyLock<AckSignal> = LazyLock::new(|| AckSignal::new());
        static BUFFER: LazyLock<RxBuffer> = LazyLock::new(|| RxBuffer::new());

        (CHANNEL.get(), SIGNAL.get(), BUFFER.get())
    }};
}

pub(crate) fn repl_rx_enable(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::RxEnableDisable(true);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_rx_disable(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::RxEnableDisable(false);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_rx_is_enabled(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<bool, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::RxIsEnabled;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;

    match result {
        RpcResult::RxIsEnabled(enabled) => Ok(enabled),
        _ => {
            unreachable!()
        }
    }
}

pub(crate) fn repl_rx_get_baud(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<u32, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::RxGetBaud;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;

    match result {
        RpcResult::RxGetBaud(baud) => Ok(baud),
        _ => {
            unreachable!()
        }
    }
}

pub(crate) fn repl_rx_set_baud(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    baud: INT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::RxSetBaud(baud as u32);
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_rx_set_mode(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    mode: &str,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let mode = match mode.to_lowercase().as_str() {
        "nmea0183" => RxMode::Nmea0183,
        "modbus" => RxMode::Modbus,
        "can" => RxMode::Can,
        _ => {
            return Err(Box::new(EvalAltResult::ErrorMismatchDataType(
                String::from("[nmea0183, modbus, can]"),
                mode.to_owned(),
                ctx.call_position(),
            )))
        }
    };

    if mode == RxMode::Modbus {
        return Err(Box::new(EvalAltResult::ErrorRuntime(
            "Sorry, Modbus Rx support isn't quite ready. Approach @nick-halt toward the end of DEF CON for updated firmware.".into(),
            ctx.call_position(),
        )));
    }

    let call = RpcCall::RxSetMode(mode);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_rx_get_mode(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<ImmutableString, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::RxGetMode;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;

    let mode = match result {
        RpcResult::RxGetMode(mode) => match mode {
            RxMode::Nmea0183 => "nmea0183",
            RxMode::Modbus => "modbus",
            RxMode::Can => "can",
        },
        _ => {
            unreachable!()
        }
    };

    Ok(ImmutableString::from(mode))
}

pub(crate) fn repl_rx_recv(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    timeout_secs: FLOAT,
) -> Result<Blob, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::RxIsEnabled;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;

    match result {
        RpcResult::RxIsEnabled(enabled) => {
            if !enabled {
                return Err(Box::new(EvalAltResult::ErrorDataRace(
                    "Rx not enabled!".to_owned(),
                    ctx.call_position(),
                )));
            }
        }
        _ => {
            unreachable!()
        }
    }

    let call = RpcCall::RxRecv(Duration::from_millis((timeout_secs * 1000.0) as u64));
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;

    match result {
        RpcResult::RxRecv(msg) => Ok(msg),
        _ => {
            unreachable!()
        }
    }
}

pub(crate) fn register_functions(
    engine: &mut Engine,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) {
    let mut module = Module::new();

    // NOTE: These are idempotent.
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_rx_is_enabled,
        "is_enabled",
        ()
    );
    register_repl_fn!(module, call_tx, result_rx, repl_rx_enable, "enable", ());
    register_repl_fn!(module, call_tx, result_rx, repl_rx_disable, "disable", ());
    register_repl_fn!(module, call_tx, result_rx, repl_rx_get_baud, "get_baud", ());
    register_repl_fn!(module, call_tx, result_rx, repl_rx_set_baud, "set_baud", (baud: INT));
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_rx_set_mode,
        "set_mode",
        (mode: &str)
    );
    register_repl_fn!(module, call_tx, result_rx, repl_rx_get_mode, "get_mode", ());
    register_repl_fn!(module, call_tx, result_rx, repl_rx_recv, "recv", (timeout_secs: FLOAT));

    engine.register_static_module("rx", module.into());
}
