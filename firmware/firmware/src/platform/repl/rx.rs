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
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel};
use rhai::{Engine, EvalAltResult, ImmutableString, Module, NativeCallContext};

// TODO: Move all channels to common.rs
#[derive(Debug, Clone)]
pub enum RxCommand {
    EnableDisable(bool),
    SetMode(RxMode),
    GetMode,
}

pub const RX_MTU: usize = 1;

pub type RxChannel = channel::Channel<CriticalSectionRawMutex, RxCommand, RX_MTU>;
pub type RxSender = channel::Sender<'static, CriticalSectionRawMutex, RxCommand, RX_MTU>;
pub type RxReceiver = channel::Receiver<'static, CriticalSectionRawMutex, RxCommand, RX_MTU>;

#[macro_export]
macro_rules! make_rx_channels {
    () => {{
        use crate::platform::repl::{common::AckSignal, rx::RxChannel};
        use embassy_sync::lazy_lock::LazyLock;

        static CHANNEL: LazyLock<RxChannel> = LazyLock::new(|| RxChannel::new());
        static SIGNAL: LazyLock<AckSignal> = LazyLock::new(|| AckSignal::new());

        (CHANNEL.get(), SIGNAL.get())
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

pub(crate) fn repl_rx_set_mode(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    mode: String,
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

pub(crate) fn register_functions(
    engine: &mut Engine,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) {
    let mut module = Module::new();

    // NOTE: These are idempotent.
    register_repl_fn!(module, call_tx, result_rx, repl_rx_enable, "enable", ());
    register_repl_fn!(module, call_tx, result_rx, repl_rx_disable, "disable", ());
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_rx_set_mode,
        "set_mode",
        (mode: String)
    );
    register_repl_fn!(module, call_tx, result_rx, repl_rx_get_mode, "get_mode", ());

    engine.register_static_module("rx", module.into());
}
