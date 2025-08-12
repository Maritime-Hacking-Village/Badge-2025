//! Differential injector/receiver RPC calls

use crate::{
    platform::repl::{
        rpc::{RpcCall, RpcCallSender, RpcResult, RpcResultReceiver},
        rpc_call,
    },
    register_repl_fn,
};
use alloc::boxed::Box;
use rhai::{Array, Dynamic, Engine, EvalAltResult, Module, NativeCallContext};

const TERM_OPEN_0: bool = false;
const TERM_OPEN_1: bool = false;
const TERM_120R_0: bool = true;
const TERM_120R_1: bool = false;
const TERM_220R_0: bool = false;
const TERM_220R_1: bool = true;
const TERM_13R_0: bool = true;
const TERM_13R_1: bool = true;

pub(crate) fn repl_trx_get_term(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<Array, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::TrxGetTerm;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;

    match result {
        RpcResult::TrxGetTerm(sel_0, sel_1) => {
            let mut term_arr = Array::new();
            term_arr.push(Dynamic::from_bool(sel_0));
            term_arr.push(Dynamic::from_bool(sel_1));
            Ok(term_arr)
        }
        _ => {
            unreachable!()
        }
    }
}

pub(crate) fn repl_trx_set_term(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    sel_0: bool,
    sel_1: bool,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::TrxSetTerm(sel_0, sel_1);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_trx_get_tie(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<bool, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::TrxGetTie;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;

    match result {
        RpcResult::TrxGetTie(tied) => Ok(tied),
        _ => {
            unreachable!()
        }
    }
}

pub(crate) fn repl_trx_set_tie(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    tied: bool,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::TrxSetTie(tied);
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
        repl_trx_get_term,
        "get_term",
        ()
    );
    register_repl_fn!(module, call_tx, result_rx, repl_trx_set_term, "set_term", (sel_0: bool, sel_1: bool));
    register_repl_fn!(module, call_tx, result_rx, repl_trx_get_tie, "get_tie", ());
    register_repl_fn!(module, call_tx, result_rx, repl_trx_set_tie, "set_tie", (tied: bool));

    module.set_var("TERM_OPEN_0", TERM_OPEN_0);
    module.set_var("TERM_OPEN_1", TERM_OPEN_1);
    module.set_var("TERM_120R_0", TERM_120R_0);
    module.set_var("TERM_120R_1", TERM_120R_1);
    module.set_var("TERM_220R_0", TERM_220R_0);
    module.set_var("TERM_220R_1", TERM_220R_1);
    module.set_var("TERM_13R_0", TERM_13R_0);
    module.set_var("TERM_13R_1", TERM_13R_1);

    engine.register_static_module("trx", module.into());
}
