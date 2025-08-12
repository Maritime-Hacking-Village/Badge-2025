//! System RPC calls

use crate::{
    platform::repl::{
        rpc::{RpcCall, RpcCallSender, RpcResult, RpcResultReceiver},
        rpc_call,
    },
    register_repl_fn,
};
use alloc::boxed::Box;
use rhai::{Array, Dynamic, Engine, EvalAltResult, Module, NativeCallContext};

pub(crate) fn repl_sao_get_direction(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<Array, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::SaoGetDirection;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;
    let mut ret = Array::new();

    match result {
        RpcResult::SaoGetDirection(dir_1, dir_2) => {
            ret.push(Dynamic::from_bool(dir_1));
            ret.push(Dynamic::from_bool(dir_2));
        }
        _ => {
            unreachable!()
        }
    }

    Ok(ret)
}

pub(crate) fn repl_sao_set_direction(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    dir_1: bool,
    dir_2: bool,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::SaoSetDirection(dir_1, dir_2);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_sao_read(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<Array, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::SaoRead;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;
    let mut ret = Array::new();

    match result {
        RpcResult::SaoRead(gpio_1, gpio_2) => {
            ret.push(Dynamic::from_bool(gpio_1));
            ret.push(Dynamic::from_bool(gpio_2));
        }
        _ => {
            unreachable!()
        }
    }

    Ok(ret)
}

pub(crate) fn repl_sao_write(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    output_1: bool,
    output_2: bool,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::SaoWrite(output_1, output_2);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn register_functions(
    engine: &mut Engine,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) {
    let mut module = Module::new();
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_sao_get_direction,
        "get_direction",
        ()
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_sao_set_direction,
        "set_direction",
        (dir_1: bool, dir_2: bool)
    );
    register_repl_fn!(module, call_tx, result_rx, repl_sao_read, "read", ());
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_sao_write,
        "write",
        (output_1: bool, output_2: bool)
    );
    engine.register_static_module("sao", module.into());
}
