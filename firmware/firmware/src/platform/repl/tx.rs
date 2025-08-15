//! Differential injector RPC calls

use crate::{
    apps::tx::{TxMode, TxWords},
    platform::repl::{
        rpc::{RpcCall, RpcCallSender, RpcResult, RpcResultReceiver},
        rpc_call,
    },
    register_repl_fn,
};
use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};
use defmt::{debug, warn};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel};
use rhai::{Blob, Engine, EvalAltResult, ImmutableString, Module, NativeCallContext, INT};

pub const L_Z0: u8 = 0b000_0_000_1;
pub const L_0V: u8 = 0b000_0_000_0;
pub const L_1V: u8 = 0b000_0_100_0;
pub const L_1V5: u8 = 0b000_0_010_0;
pub const L_2V: u8 = 0b000_0_110_0;
pub const L_2V5: u8 = 0b000_0_001_0;
pub const L_3V: u8 = 0b000_0_101_0;
pub const L_3V5: u8 = 0b000_0_011_0;
pub const L_4V: u8 = 0b000_0_111_0;

pub const H_Z0: u8 = 0b000_1_000_0;
pub const H_0V: u8 = 0b000_0_000_0;
pub const H_1V: u8 = 0b100_0_000_0;
pub const H_1V5: u8 = 0b010_0_000_0;
pub const H_2V: u8 = 0b110_0_000_0;
pub const H_2V5: u8 = 0b001_0_000_0;
pub const H_3V: u8 = 0b101_0_000_0;
pub const H_3V5: u8 = 0b011_0_000_0;
pub const H_4V: u8 = 0b111_0_000_0;

pub const LOW_Z: u8 = !(H_Z0 | L_Z0);

// TODO GetEnabled?
#[derive(Debug, Clone)]
pub enum TxCommand {
    EnableDisable(bool),
    SetBaud(u32),
    GetMode,
    SetMode(TxMode),
    Send(TxWords),
}

pub const TX_MTU: usize = 1;

pub type TxChannel = channel::Channel<CriticalSectionRawMutex, TxCommand, TX_MTU>;
pub type TxSender = channel::Sender<'static, CriticalSectionRawMutex, TxCommand, TX_MTU>;
pub type TxReceiver = channel::Receiver<'static, CriticalSectionRawMutex, TxCommand, TX_MTU>;

#[macro_export]
macro_rules! make_tx_channels {
    () => {{
        use crate::platform::repl::{common::AckSignal, tx::TxChannel};
        use embassy_sync::lazy_lock::LazyLock;

        static CHANNEL: LazyLock<TxChannel> = LazyLock::new(|| TxChannel::new());
        static SIGNAL: LazyLock<AckSignal> = LazyLock::new(|| AckSignal::new());

        (CHANNEL.get(), SIGNAL.get())
    }};
}

pub(crate) fn repl_tx_enable(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::TxEnableDisable(true);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_tx_disable(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::TxEnableDisable(false);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_tx_set_baud(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    baud: INT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    if baud <= 0 {
        return Err(Box::new(EvalAltResult::ErrorArithmetic(
            "".to_owned(),
            ctx.call_position(),
        )));
    }

    let call = RpcCall::TxSetBaud(baud as u32);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_tx_set_mode(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    mode: String,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let mode = match mode.to_lowercase().as_str() {
        "inject" => TxMode::Inject,
        "can" => TxMode::Can,
        _ => {
            return Err(Box::new(EvalAltResult::ErrorMismatchDataType(
                String::from("[inject, can]"),
                mode.to_owned(),
                ctx.call_position(),
            )))
        }
    };
    let call = RpcCall::TxSetMode(mode);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_tx_get_mode(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<ImmutableString, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::TxGetMode;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;

    let mode = match result {
        RpcResult::TxGetMode(mode) => match mode {
            TxMode::Inject => "inject",
            TxMode::Can => "can",
        },
        _ => {
            unreachable!()
        }
    };

    Ok(ImmutableString::from(mode))
}

fn bytes_to_u32(mut bytes: Vec<u8>) -> Vec<u32> {
    if bytes.len() == 0 {
        return Vec::new();
    }

    while bytes.len() % 4 != 0 {
        debug!("Adding new nil byte!");
        bytes.push(0xFF);
    }

    bytes
        .chunks(4)
        .filter(|chunk| chunk.len() == 4)
        .map(|chunk| {
            ((chunk[0] as u32) << 24)
                | ((chunk[1] as u32) << 16)
                | ((chunk[2] as u32) << 8)
                | (chunk[3] as u32)
        })
        .collect()
}

pub(crate) fn repl_tx_send(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    data: Blob,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::TxGetMode;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;
    warn!("0 {:?}", result);

    let words = match result {
        RpcResult::TxGetMode(TxMode::Inject) => TxWords::Inject(data),
        RpcResult::TxGetMode(TxMode::Can) => TxWords::Can(bytes_to_u32(data)),
        _ => unreachable!(),
    };

    let call = RpcCall::TxSend(words);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn register_functions(
    engine: &mut Engine,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) {
    let mut module = Module::new();

    module.set_var("H_Z0", H_Z0 as INT);
    module.set_var("H_0V", H_0V as INT);
    module.set_var("H_1V", H_1V as INT);
    module.set_var("H_1V5", H_1V5 as INT);
    module.set_var("H_2V", H_2V as INT);
    module.set_var("H_2V5", H_2V5 as INT);
    module.set_var("H_3V", H_3V as INT);
    module.set_var("H_3V5", H_3V5 as INT);
    module.set_var("H_4V", H_4V as INT);

    module.set_var("L_Z0", L_Z0 as INT);
    module.set_var("L_0V", L_0V as INT);
    module.set_var("L_1V", L_1V as INT);
    module.set_var("L_1V5", L_1V5 as INT);
    module.set_var("L_2V", L_2V as INT);
    module.set_var("L_2V5", L_2V5 as INT);
    module.set_var("L_3V", L_3V as INT);
    module.set_var("L_3V5", L_3V5 as INT);
    module.set_var("L_4V", L_4V as INT);

    module.set_var("LOW_Z", LOW_Z as INT);

    register_repl_fn!(module, call_tx, result_rx, repl_tx_enable, "enable", ());
    register_repl_fn!(module, call_tx, result_rx, repl_tx_disable, "disable", ());
    register_repl_fn!(module, call_tx, result_rx, repl_tx_set_baud, "set_baud", (baud: INT));
    register_repl_fn!(module, call_tx, result_rx, repl_tx_get_mode, "get_mode", ());
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_tx_set_mode,
        "set_mode",
        (mode: String)
    );
    register_repl_fn!(module, call_tx, result_rx, repl_tx_send, "send", (data: Blob));

    engine.register_static_module("tx", module.into());
}
