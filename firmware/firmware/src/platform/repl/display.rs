use crate::{
    platform::repl::{
        rpc::{RpcCall, RpcCallSender, RpcResultReceiver},
        rpc_call,
    },
    register_repl_fn,
};
use alloc::{boxed::Box, string::String};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel};
use embedded_graphics::pixelcolor::Rgb565;
use rhai::{Engine, EvalAltResult, Module, NativeCallContext, FLOAT, INT};

#[derive(Debug, Clone)]
pub enum DisplayCommand {
    ConsoleWrite(String),
    SetPixel(u16, u16, Rgb565),
    FillRegion(u16, u16, u16, u16, Rgb565),
    Clear,
    Flush,
}

pub const DISP_MTU: usize = 1;

pub type DisplayChannel = channel::Channel<CriticalSectionRawMutex, DisplayCommand, DISP_MTU>;
pub type DisplaySender =
    channel::Sender<'static, CriticalSectionRawMutex, DisplayCommand, DISP_MTU>;
pub type DisplayReceiver =
    channel::Receiver<'static, CriticalSectionRawMutex, DisplayCommand, DISP_MTU>;

#[macro_export]
macro_rules! make_display_channel {
    () => {{
        use crate::platform::repl::display::DisplayChannel;
        use embassy_sync::lazy_lock::LazyLock;

        static CHANNEL: LazyLock<DisplayChannel> = LazyLock::new(|| DisplayChannel::new());

        CHANNEL.get()
    }};
}

pub const DISP_MAX_X: usize = 320;
pub const DISP_MAX_Y: usize = 170;

pub(crate) fn repl_display_set_backlight_int(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    percent: INT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    if percent < 0 || percent > 100 {
        return Err(Box::new(EvalAltResult::ErrorArithmetic(
            String::from("Percent must be in [0, 100]."),
            ctx.call_position(),
        )));
    }

    let call = RpcCall::DisplaySetBacklight(percent as u8);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_display_set_backlight_float(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    percent: FLOAT,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    if percent < 0.0 || percent > 100.0 {
        return Err(Box::new(EvalAltResult::ErrorArithmetic(
            String::from("Percent must be in [0, 100]."),
            ctx.call_position(),
        )));
    }

    let call = RpcCall::DisplaySetBacklight(percent as u8);
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_display_set_pixel(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    x: INT,
    y: INT,
    r: INT,
    g: INT,
    b: INT,
) -> Result<(), Box<EvalAltResult>> {
    if x < 0 || x > (DISP_MAX_X - 1) as INT {
        return Err(EvalAltResult::ErrorArrayBounds(DISP_MAX_X, x, ctx.call_position()).into());
    } else if y < 0 || y > (DISP_MAX_Y - 1) as INT {
        return Err(EvalAltResult::ErrorArrayBounds(DISP_MAX_Y, y, ctx.call_position()).into());
    }

    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::DisplaySetPixel(x as u16, y as u16, Rgb565::new(r as u8, g as u8, b as u8));
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_display_fill_region(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    sx: INT,
    ex: INT,
    sy: INT,
    ey: INT,
    r: INT,
    g: INT,
    b: INT,
) -> Result<(), Box<EvalAltResult>> {
    if sx < 0 || sx > (DISP_MAX_X - 1) as INT {
        return Err(EvalAltResult::ErrorArrayBounds(DISP_MAX_X, sx, ctx.call_position()).into());
    } else if ex < 0 || ex > (DISP_MAX_X - 1) as INT {
        return Err(EvalAltResult::ErrorArrayBounds(DISP_MAX_X, ex, ctx.call_position()).into());
    } else if sy < 0 || sy > (DISP_MAX_Y - 1) as INT {
        return Err(EvalAltResult::ErrorArrayBounds(DISP_MAX_Y, sy, ctx.call_position()).into());
    } else if ey < 0 || sy > (DISP_MAX_Y - 1) as INT {
        return Err(EvalAltResult::ErrorArrayBounds(DISP_MAX_Y, ey, ctx.call_position()).into());
    }

    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::DisplayFillRegion(
        sx as u16,
        ex as u16,
        sy as u16,
        ey as u16,
        Rgb565::new(r as u8, g as u8, b as u8),
    );
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_display_clear(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::DisplayClear;
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_display_flush(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::DisplayFlush;
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_display_reset(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::DisplayReset;
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
        repl_display_set_backlight_int,
        "set_backlight",
        (percent: INT)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_display_set_backlight_float,
        "set_backlight",
        (percent: FLOAT)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_display_set_pixel,
        "set_pixel",
        (x: INT, y: INT, r: INT, g: INT, b: INT)
    );
    register_repl_fn!(
        module,
        call_tx,
        result_rx,
        repl_display_fill_region,
        "fill_region",
        (sx: INT, ex: INT, sy: INT, ey: INT, r: INT, g: INT, b: INT)
    );
    register_repl_fn!(module, call_tx, result_rx, repl_display_clear, "clear", ());
    register_repl_fn!(module, call_tx, result_rx, repl_display_flush, "flush", ());
    register_repl_fn!(module, call_tx, result_rx, repl_display_reset, "reset", ());
    engine.register_static_module("display", module.into());
}
