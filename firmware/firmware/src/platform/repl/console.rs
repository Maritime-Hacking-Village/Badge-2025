use crate::{
    platform::repl::{
        rpc::{RpcCall, RpcCallSender, RpcResultReceiver},
        rpc_call,
    },
    register_repl_fn,
};
use alloc::{borrow::ToOwned, boxed::Box};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pipe};
use rhai::{Array, Engine, EvalAltResult, Module, NativeCallContext};
use static_cell::StaticCell;

pub const CONSOLE_MTU: usize = 128;

pub type ConsolePipe = pipe::Pipe<CriticalSectionRawMutex, CONSOLE_MTU>;
pub type ConsoleWriter = pipe::Writer<'static, CriticalSectionRawMutex, CONSOLE_MTU>;
pub type ConsoleReader = pipe::Reader<'static, CriticalSectionRawMutex, CONSOLE_MTU>;

pub static CONSOLE_PIPE: StaticCell<ConsolePipe> = StaticCell::new();

// pub const ANSI_HOME: &str = "\x1b[H";

// pub const ANSI_RESET: &str = "\x1b[0m";
// pub const ANSI_BOLD: &str = "\x1b[";
// pub const ANSI_DIM: &str = "\x1b[";
// pub const ANSI_ITALIC: &str = "\x1b[";
// pub const ANSI_UNDERLINE: &str = "\x1b[";
// pub const ANSI_BLINKING: &str = "\x1b[";
// pub const ANSI_INVERSE: &str = "\x1b[";
// pub const ANSI_HIDDEN: &str = "\x1b[";
// pub const ANSI_STRIKE: &str = "\x1b[";

// pub const ANSI_BLACK_FORE: &str = "\x1b[30";
// pub const ANSI_RED_FORE: &str = "\x1b[31";
// pub const ANSI_GREEN_FORE: &str = "\x1b[32";
// pub const ANSI_YELLOW_FORE: &str = "\x1b[33";
// pub const ANSI_BLUE_FORE: &str = "\x1b[34";
// pub const ANSI_MAGENTA_FORE: &str = "\x1b[35";
// pub const ANSI_CYAN_FORE: &str = "\x1b[36";
// pub const ANSI_WHITE_FORE: &str = "\x1b[37";
// pub const ANSI_DEFAULT_FORE: &str = "\x1b[39";

// pub const ANSI_BLACK_BACK: &str = "\x1b[40";
// pub const ANSI_RED_BACK: &str = "\x1b[41";
// pub const ANSI_GREEN_BACK: &str = "\x1b[42";
// pub const ANSI_YELLOW_BACK: &str = "\x1b[43";
// pub const ANSI_BLUE_BACK: &str = "\x1b[44";
// pub const ANSI_MAGENTA_BACK: &str = "\x1b[45";
// pub const ANSI_CYAN_BACK: &str = "\x1b[46";
// pub const ANSI_WHITE_BACK: &str = "\x1b[47";
// pub const ANSI_DEFAULT_BACK: &str = "\x1b[49";

pub(crate) fn repl_console_write(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    text: &str,
) -> Result<(), Box<EvalAltResult>> {
    // Construct the RpcCall and send it non-blocking (errors if unable to send).
    let call = RpcCall::ConsoleWrite(text.to_owned());
    let _result = rpc_call(&ctx, call_tx, result_rx, call)?;

    Ok(())
}

pub(crate) fn repl_console_write_array(
    ctx: &NativeCallContext,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    texts_dyn: Array,
) -> Result<(), Box<EvalAltResult>> {
    for text_dyn in texts_dyn {
        let text = match text_dyn.into_string().map_err(|err| {
            Box::new(EvalAltResult::ErrorMismatchDataType(
                "str".to_owned(),
                err.to_owned(),
                ctx.call_position(),
            ))
        }) {
            Ok(text) => text,
            Err(err) => {
                return Err(err);
            }
        };

        repl_console_write(ctx, call_tx, result_rx, text.as_str())?;
    }

    Ok(())
}

pub(crate) fn register_functions(
    engine: &mut Engine,
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
) {
    let mut module = Module::new();

    // module.set_var("HOME", ANSI_HOME);

    // module.set_var("RESET", ANSI_RESET);
    // module.set_var("BOLD", ANSI_BOLD);
    // module.set_var("DIM", ANSI_DIM);
    // module.set_var("ITALIC", ANSI_ITALIC);
    // module.set_var("UNDERLINE", ANSI_UNDERLINE);
    // module.set_var("BLINKING", ANSI_BLINKING);
    // module.set_var("INVERSE", ANSI_INVERSE);
    // module.set_var("HIDDEN", ANSI_HIDDEN);
    // module.set_var("STRIKE", ANSI_STRIKE);

    // module.set_var("BLACK_FORE", ANSI_BLACK_FORE);
    // module.set_var("RED_FORE", ANSI_RED_FORE);
    // module.set_var("GREEN_FORE", ANSI_GREEN_FORE);
    // module.set_var("YELLOW_FORE", ANSI_YELLOW_FORE);
    // module.set_var("BLUE_FORE", ANSI_BLUE_FORE);
    // module.set_var("MAGENTA_FORE", ANSI_MAGENTA_FORE);
    // module.set_var("CYAN_FORE", ANSI_CYAN_FORE);
    // module.set_var("WHITE_FORE", ANSI_WHITE_FORE);
    // module.set_var("DEFAULT_FORE", ANSI_DEFAULT_FORE);

    // module.set_var("BLACK_BACK", ANSI_BLACK_BACK);
    // module.set_var("RED_BACK", ANSI_RED_BACK);
    // module.set_var("GREEN_BACK", ANSI_GREEN_BACK);
    // module.set_var("YELLOW_BACK", ANSI_YELLOW_BACK);
    // module.set_var("BLUE_BACK", ANSI_BLUE_BACK);
    // module.set_var("MAGENTA_BACK", ANSI_MAGENTA_BACK);
    // module.set_var("CYAN_BACK", ANSI_CYAN_BACK);
    // module.set_var("WHITE_BACK", ANSI_WHITE_BACK);
    // module.set_var("DEFAULT_BACK", ANSI_DEFAULT_BACK);

    register_repl_fn!(module, call_tx, result_rx, repl_console_write, "write", (text: &str));
    register_repl_fn!(module, call_tx, result_rx, repl_console_write_array, "write", (texts_dyn: Array));

    engine.register_static_module("console", module.into());
}
