//! System RPC calls

use crate::{
    platform::{
        i2c_io_expander::models::tcal9539::TCAL9539,
        repl::{
            rpc::{RpcCall, RpcCallSender, RpcResult, RpcResultReceiver},
            rpc_call,
        },
    },
    register_repl_fn,
};
use alloc::boxed::Box;
use rhai::{Engine, EvalAltResult, Map, Module, NativeCallContext};

pub(crate) fn repl_input_read(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<Map, Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::InputRead;
    let result = rpc_call(&ctx, call_tx, result_rx, call)?;
    let mut ret = Map::new();

    match result {
        RpcResult::InputRead(inputs) => {
            ret.insert("up".into(), (!inputs[TCAL9539::PIN_JOY_UP]).into());
            ret.insert("right".into(), (!inputs[TCAL9539::PIN_JOY_RIGHT]).into());
            ret.insert("down".into(), (!inputs[TCAL9539::PIN_JOY_DOWN]).into());
            ret.insert("center".into(), (!inputs[TCAL9539::PIN_JOY_CENTER]).into());
            ret.insert("left".into(), (!inputs[TCAL9539::PIN_JOY_LEFT]).into());
            ret.insert("a".into(), (!inputs[TCAL9539::PIN_BUTTON_A]).into());
            ret.insert("b".into(), (!inputs[TCAL9539::PIN_BUTTON_B]).into());
            // ret.insert("disp_rst".into(), inputs[TCAL9539::PIN_DISP_RST].into());
            // ret.insert("sao_gpio_1".into(), inputs[TCAL9539::PIN_SAO_1].into());
            // ret.insert(
            //     "tx_disconnect".into(),
            //     inputs[TCAL9539::PIN_TX_DISCONNECT].into(),
            // );
            // ret.insert("tx_disable".into(), inputs[TCAL9539::PIN_TX_DISABLE].into());
            // ret.insert(
            //     "can_disconnect".into(),
            //     inputs[TCAL9539::PIN_CAN_DISCONNECT].into(),
            // );
            // ret.insert("sd_cd".into(), inputs[TCAL9539::PIN_SD_CD].into());
            // ret.insert(
            //     "term_sel_0".into(),
            //     inputs[TCAL9539::PIN_RX_TERM_SEL0].into(),
            // );
            // ret.insert(
            //     "term_sel_1".into(),
            //     inputs[TCAL9539::PIN_RX_TERM_SEL1].into(),
            // );
            // ret.insert("rx_tx_tie".into(), inputs[TCAL9539::PIN_RX_TX_TIE].into());
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
    register_repl_fn!(module, call_tx, result_rx, repl_input_read, "read", ());
    engine.register_static_module("input", module.into());
}
