//! System RPC calls

use crate::{
    platform::repl::{
        rpc::{RpcCall, RpcCallSender, RpcResult, RpcResultReceiver},
        rpc_call,
    },
    register_repl_fn,
};
use alloc::boxed::Box;
use rhai::{Engine, EvalAltResult, Map, Module, NativeCallContext, INT};

pub(crate) fn repl_batt_status(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<Map, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::BattStatus;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;
    let mut ret = Map::new();

    match result {
        RpcResult::BattStatus(status) => {
            // TODO: Proper Rhai CustomType integration.
            ret.insert(
                "vbus_stat".into(),
                ((status.reg0b.vbus_stat as u8) as INT).into(),
            );
            ret.insert(
                "chrg_stat".into(),
                ((status.reg0b.chrg_stat as u8) as INT).into(),
            );
            ret.insert("pg_stat".into(), status.reg0b.pg_stat.into());
            ret.insert("sdp_stat".into(), status.reg0b.sdp_stat.into());
            ret.insert("vsys_stat".into(), status.reg0b.vsys_stat.into());

            ret.insert("watchdog_fault".into(), status.reg0c.watchdog_fault.into());
            ret.insert("boost_fault".into(), status.reg0c.boost_fault.into());
            ret.insert(
                "chrg_fault".into(),
                ((status.reg0c.chrg_fault as u8) as INT).into(),
            );
            ret.insert("bat_fault".into(), status.reg0c.bat_fault.into());
            ret.insert(
                "ntc_fault".into(),
                ((status.reg0c.ntc_fault as u8) as INT).into(),
            );

            ret.insert("therm_stat".into(), status.reg0e.therm_stat.into());
            ret.insert("batv".into(), (status.reg0e.batv.get_value() as INT).into());

            ret.insert("sysv".into(), (status.reg0f.sysv.get_value() as INT).into());

            ret.insert(
                "tspct".into(),
                (status.reg10.tspct.get_value() as INT).into(),
            );

            ret.insert("vbus_gd".into(), status.reg11.vbus_gd.into());
            ret.insert(
                "vbusv".into(),
                (status.reg11.vbusv.get_value() as INT).into(),
            );

            ret.insert(
                "ichgr".into(),
                (status.reg12.ichgr.get_value() as INT).into(),
            );

            ret.insert("vdpm_stat".into(), status.reg13.vdpm_stat.into());
            ret.insert("idpm_stat".into(), status.reg13.idpm_stat.into());
            ret.insert(
                "idpm_lim".into(),
                (status.reg13.idpm_lim.get_value() as INT).into(),
            );
        }
        _ => {
            unreachable!()
        }
    }

    Ok(ret)
}

pub(crate) fn register_functions(
    engine: &mut Engine,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) {
    let mut module = Module::new();
    register_repl_fn!(module, call_tx, result_rx, repl_batt_status, "status", ());
    engine.register_static_module("batt", module.into());
}
