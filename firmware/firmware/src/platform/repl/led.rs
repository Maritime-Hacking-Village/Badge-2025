use crate::{
    platform::repl::{
        rpc::{RpcCall, RpcCallSender, RpcResultReceiver},
        rpc_call,
    },
    register_repl_fn,
};
use alloc::boxed::Box;
use defmt::debug;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel};
use rhai::{Engine, EvalAltResult, Module, NativeCallContext, INT};
use smart_leds::RGB8;

pub const LEDS_MTU: usize = 1;

pub type LedChannel = channel::Channel<CriticalSectionRawMutex, (usize, RGB8), LEDS_MTU>;
pub type LedSender = channel::Sender<'static, CriticalSectionRawMutex, (usize, RGB8), LEDS_MTU>;
pub type LedReceiver = channel::Receiver<'static, CriticalSectionRawMutex, (usize, RGB8), LEDS_MTU>;

#[macro_export]
macro_rules! make_leds_channel {
    () => {{
        use crate::platform::repl::led::LedChannel;

        use embassy_sync::lazy_lock::LazyLock;

        static CHANNEL: LazyLock<LedChannel> = LazyLock::new(|| LedChannel::new());

        CHANNEL.get()
    }};
}

pub(crate) fn repl_led_set(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    i: INT,
    r: INT,
    g: INT,
    b: INT,
) -> Result<(), Box<EvalAltResult>> {
    debug!("repl_led_set {} ({}, {}, {})", i, r, g, b);

    // TODO more consts
    if i >= 9 {
        return Err(Box::new(EvalAltResult::ErrorArrayBounds(
            9_usize,
            i,
            ctx.call_position(),
        )));
    }

    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::LedSet(i as usize, RGB8::new(r as u8, g as u8, b as u8));
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
        repl_led_set,
        "set",
        (i: INT, r: INT, g: INT, b: INT)
    );
    engine.register_static_module("led", module.into());
}
