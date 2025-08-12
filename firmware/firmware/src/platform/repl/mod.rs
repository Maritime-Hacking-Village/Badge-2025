pub mod accel;
pub mod batt;
pub mod can;
pub mod common;
pub mod console;
pub mod display;
pub mod input;
pub mod led;
pub mod nmea2000;
pub mod rpc;
pub mod rx;
pub mod sao;
pub mod sys;
pub mod trx;
pub mod tx;

use crate::platform::repl::rpc::{
    CallId, RpcCall, RpcCallSender, RpcError, RpcResult, RpcResultReceiver, ToEndpoint,
};
use alloc::boxed::Box;
use defmt::{trace, warn, ToString};
use rhai::{Engine, EvalAltResult, NativeCallContext, Position};

#[macro_export]
macro_rules! send_sync_block {
    ($tx:ident,$val:expr,$err_msg:expr) => {{
        let mut stuck = false;

        while $tx.try_send($val.clone()).is_err() {
            if !stuck {
                trace!("----- BEGIN {:?} -----", $err_msg);
                stuck = true;
            }

            core::hint::spin_loop();
        }

        if stuck {
            trace!("----- END {:?} -----", $err_msg);
        }
    }};
}

#[macro_export]
macro_rules! send_sync_noblock {
    ($tx:ident,$val:expr,$pos:expr,$err_msg:expr) => {
        $tx.try_send($val)
            .map_err(|_| Box::new(EvalAltResult::ErrorDataRace($err_msg.to_string(), $pos)))?
    };
}

#[macro_export]
macro_rules! recv_sync_block {
    ($rx:ident,$err_msg:expr) => {{
        let mut stuck = false;

        loop {
            let res = $rx.try_receive();

            if let Ok(val) = res {
                if stuck {
                    trace!("----- END {:?} : {:?} -----", $err_msg, val);
                }

                break val;
            }

            if !stuck {
                trace!("----- BEGIN {:?} -----", $err_msg);
                stuck = true;
            }

            core::hint::spin_loop();
        }
    }};
}

#[macro_export]
macro_rules! recv_sync_noblock {
    ($rx:ident,$pos:expr,$err_msg:expr) => {
        $rx.try_recv()
            .map_err(|_| Box::new(EvalAltResult::ErrorDataRace($err_msg.to_string(), $pos)))?
    };
}

#[macro_export]
macro_rules! register_repl_fn {
    (
        $module:ident,
        $call_channel:ident,
        $result_channel:ident,
        $fn_name:ident,
        $fn_str:literal,
        ( $($arg_name:ident : $arg_ty:ty),* $(,)? )
    ) => {
        $module.set_native_fn(
            $fn_str,
            move |ctx: NativeCallContext, $($arg_name: $arg_ty),*| {
                $fn_name(
                    &ctx,
                    $call_channel,
                    $result_channel,
                    $($arg_name),*
                )
            },
        );
    };
}

#[macro_export]
macro_rules! register_repl_fn_no_rpc {
    (
        $module:ident,
        $fn_name:ident,
        $fn_str:literal,
        ( $($arg_name:ident : $arg_ty:ty),* $(,)? )
    ) => {
        $module.set_native_fn(
            $fn_str,
            move |ctx: NativeCallContext, $($arg_name: $arg_ty),*| {
                $fn_name(
                    &ctx,
                    $($arg_name),*
                )
            },
        );
    };
}

// #[macro_export]
// macro_rules! register_repl_fn_with_ptr {
//     (
//         $module:ident,
//         $call_channel:ident,
//         $result_channel:ident,
//         $fn_name:ident,
//         $fn_str:literal,
//         ( $($arg_name:ident : $arg_ty:ty),* $(,)? )
//     ) => {
//         $module.set_native_fn(
//             $fn_str,
//             move |ctx: &NativeCallContext, $($arg_name: $arg_ty),*| {
//                 $fn_name(
//                     &ctx,
//                     $call_channel,
//                     $result_channel,
//                     $($arg_name),*
//                 )
//             },
//         );
//     };
// }

pub(crate) fn rpc_call(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    call: RpcCall,
) -> Result<RpcResult, Box<EvalAltResult>> {
    // Send the Call non-blocking (errors if unable to send).
    let call_endpoint = call.to_endpoint();
    warn!("Sending {:?} to RPC Call channel.", call_endpoint);
    send_sync_noblock!(
        call_tx,
        call,
        ctx.call_position(),
        "Unable to send to RPC Call channel!"
    );

    // Wait for the dispatched async function to complete.
    // Assert the equality of the CallId and RpcEndpoint, then propagate any errors.
    warn!("Polling for RPC result.");
    let (_call_id, maybe_result): (CallId, Result<RpcResult, RpcError>) =
        recv_sync_block!(result_rx, "Unable to receive from RPC Result channel!");
    let result = maybe_result.map_err(|err| err.with_position(ctx.call_position()))?;
    assert_eq!(call_endpoint, result.to_endpoint());

    Ok(result)
}

#[allow(unused)]
pub(crate) fn rpc_call_no_ctx(
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    call: RpcCall,
) -> Result<RpcResult, Box<EvalAltResult>> {
    // Send the Call non-blocking (errors if unable to send).
    let call_endpoint = call.to_endpoint();
    warn!("Sending {:?} to RPC Call channel.", call_endpoint);
    send_sync_noblock!(
        call_tx,
        call,
        Position::START,
        "Unable to send to RPC Call channel!"
    );

    // Wait for the dispatched async function to complete.
    // Assert the equality of the CallId and RpcEndpoint, then propagate any errors.
    warn!("Polling for RPC result.");
    let (call_id, maybe_result): (CallId, Result<RpcResult, RpcError>) =
        recv_sync_block!(result_rx, "Unable to receive from RPC Result channel!");
    let result = maybe_result.map_err(|err| err.with_position(Position::START))?;
    assert_eq!(call_endpoint, result.to_endpoint());

    Ok(result)
}

#[allow(unused)]
pub(crate) async fn rpc_call_async(
    ctx: &NativeCallContext<'_>,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    call: RpcCall,
) -> Result<RpcResult, Box<EvalAltResult>> {
    // Send the Call non-blocking (errors if unable to send).
    let call_endpoint = call.to_endpoint();
    warn!("Sending {:?} to RPC Call channel.", call_endpoint);
    call_tx.send(call).await;

    // Wait for the dispatched async function to complete.
    // Assert the equality of the CallId and RpcEndpoint, then propagate any errors.
    warn!("Polling for RPC result.");
    let (_call_id, maybe_result): (CallId, Result<RpcResult, RpcError>) = result_rx.receive().await;
    let result = maybe_result.map_err(|err| err.with_position(ctx.call_position()))?;
    assert_eq!(call_endpoint, result.to_endpoint());

    Ok(result)
}

pub(crate) async fn rpc_call_async_no_ctx(
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    call: RpcCall,
) -> Result<RpcResult, Box<EvalAltResult>> {
    // Send the Call non-blocking (errors if unable to send).
    let call_endpoint = call.to_endpoint();
    warn!("Sending {:?} to RPC Call channel.", call_endpoint);
    call_tx.send(call).await;

    // Wait for the dispatched async function to complete.
    // Assert the equality of the CallId and RpcEndpoint, then propagate any errors.
    warn!("Polling for RPC result.");
    let (_call_id, maybe_result): (CallId, Result<RpcResult, RpcError>) = result_rx.receive().await;
    let result = maybe_result.map_err(|err| err.with_position(Position::START))?;
    assert_eq!(call_endpoint, result.to_endpoint());

    Ok(result)
}

pub fn make_engine(call_tx: RpcCallSender, result_rx: RpcResultReceiver) -> Engine {
    let mut engine = Engine::new();

    sys::register_functions(&mut engine, call_tx, result_rx);
    input::register_functions(&mut engine, call_tx, result_rx);
    sao::register_functions(&mut engine, call_tx, result_rx);
    display::register_functions(&mut engine, call_tx, result_rx);
    console::register_functions(&mut engine, call_tx, result_rx);
    led::register_functions(&mut engine, call_tx, result_rx);
    accel::register_functions(&mut engine, call_tx, result_rx);
    batt::register_functions(&mut engine, call_tx, result_rx);
    trx::register_functions(&mut engine, call_tx, result_rx);
    tx::register_functions(&mut engine, call_tx, result_rx);
    rx::register_functions(&mut engine, call_tx, result_rx);
    can::register_functions(&mut engine, call_tx, result_rx);
    nmea2000::register_functions(&mut engine, call_tx, result_rx);

    engine
}
