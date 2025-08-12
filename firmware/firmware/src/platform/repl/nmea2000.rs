//! Differential injector RPC calls

use crate::platform::repl::rpc::{RpcCallSender, RpcResultReceiver};
use alloc::boxed::Box;
use rhai::{Blob, Engine, EvalAltResult, NativeCallContext};

pub(crate) fn repl_nmea2000_encode(ctx: &NativeCallContext) -> Result<Blob, Box<EvalAltResult>> {
    Ok(Blob::new())
}

pub(crate) fn register_functions(
    engine: &mut Engine,
    _call_tx: RpcCallSender,
    _result_rx: RpcResultReceiver,
) {
    // let mut module = Module::new();
    // module.set_native_fn("encode", repl_nmea2000_encode);
    // engine.register_static_module("nmea2000", module.into());
}
