//! System RPC calls

use crate::{
    platform::repl::{
        rpc::{RpcCall, RpcCallSender, RpcResult, RpcResultReceiver},
        rpc_call,
    },
    register_repl_fn, register_repl_fn_no_rpc,
};
use alloc::boxed::Box;
use defmt::format;
use embassy_rp::clocks::clk_sys_freq;
use embassy_time::{Duration, Instant};
use rhai::{Array, Blob, Dynamic, Engine, EvalAltResult, Module, NativeCallContext, FLOAT, INT};

#[no_mangle]
#[used]
static SECRET_FLAG: &str = "MHV{JustTryAgainG}";
const CTRL_FLAG: &str = r#"      ./-\*
  .  (  0 O) MHV{AssumingDirectControl}
   \_ ` - ,   _/
-.___'.) ( ,-'
     '-.O.'-../\../
 ./\/\/ | \_.-._
        ;
     ._/      au
"#;

pub(crate) fn repl_sys_heap(ctx: &NativeCallContext) -> Result<Array, Box<EvalAltResult>> {
    let mut arr = Array::new();
    arr.push(Dynamic::from_int(crate::HEAP.used() as INT));
    arr.push(Dynamic::from_int(crate::HEAP.free() as INT));
    Ok(arr)
}

pub(crate) fn repl_sys_time(ctx: &NativeCallContext) -> Result<FLOAT, Box<EvalAltResult>> {
    Ok((Instant::now().as_millis() as FLOAT) / 1000.0)
}

pub(crate) fn repl_sys_ticks(ctx: &NativeCallContext) -> Result<INT, Box<EvalAltResult>> {
    Ok(Instant::now().as_ticks() as i64)
}

pub(crate) fn repl_sys_sleep(
    ctx: &NativeCallContext,
    secs: FLOAT,
) -> Result<(), Box<EvalAltResult>> {
    embassy_time::block_for(Duration::from_millis((secs * 1000.0) as u64));
    Ok(())
}

pub(crate) fn repl_sys_sleep_int(
    ctx: &NativeCallContext,
    secs: INT,
) -> Result<(), Box<EvalAltResult>> {
    embassy_time::block_for(Duration::from_millis(((secs as f64) * 1000.0) as u64));
    Ok(())
}

pub(crate) fn repl_sys_random(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    bytes: INT,
) -> Result<Blob, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::SysRandom(bytes as u8);
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;

    match result {
        RpcResult::SysRandom(values) => Ok(values),
        _ => {
            unreachable!()
        }
    }
}
pub(crate) fn repl_sys_assume_control(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::SysAssumeControl;
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    for line in CTRL_FLAG.lines() {
        ctx.engine()
            .eval_expression::<()>(&format!("print(#\"{}\"#)", line))?;
    }

    Ok(())
}

pub(crate) fn repl_sys_release_control(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::SysReleaseControl;
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn register_functions(
    engine: &mut Engine,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) {
    let mut module = Module::new();

    module.set_var("CLOCK_FREQ", clk_sys_freq() as INT);

    register_repl_fn_no_rpc!(module, repl_sys_heap, "heap", ());
    register_repl_fn_no_rpc!(module, repl_sys_time, "time", ());
    register_repl_fn_no_rpc!(module, repl_sys_ticks, "ticks", ());
    register_repl_fn_no_rpc!(module, repl_sys_sleep, "sleep", (secs: FLOAT));
    register_repl_fn_no_rpc!(module, repl_sys_sleep_int, "sleep", (secs: INT));
    register_repl_fn!(module, call_tx, result_rx, repl_sys_random, "random", (bytes: INT));
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_sys_assume_control,
        "assume_control",
        ()
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_sys_release_control,
        "release_control",
        ()
    );

    engine.register_static_module("sys", module.into());
}
