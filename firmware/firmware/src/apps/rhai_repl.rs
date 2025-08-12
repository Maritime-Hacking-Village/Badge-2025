use crate::platform::repl::{
    rpc::{RpcCall, RpcCallSender, RpcResultReceiver},
    rpc_call_async_no_ctx,
};
use alloc::{
    format,
    string::{String, ToString},
};
use defmt::{debug, warn, Format};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel};
use embassy_time::Instant;
use itertools::Itertools;
use rhai::{debugger::DebuggerCommand, Dynamic, Engine, EvalAltResult, Scope, AST};

#[derive(Debug, Format, Clone, Copy)]
pub enum ReplFlowControl<T> {
    Input(T),
    Break,
}

#[derive(Debug, Format, Clone, Copy)]
pub enum ReplPrintControl<T> {
    Done(T),
    Continue(T),
    Debug(T),
}

impl<T> ReplPrintControl<T> {
    #[allow(unused)]
    pub fn into_inner(self) -> T {
        match self {
            ReplPrintControl::Done(value) => value,
            ReplPrintControl::Continue(value) => value,
            ReplPrintControl::Debug(value) => value,
        }
    }
}

pub const REPL_IN_MTU: usize = 4;
pub const REPL_OUT_MTU: usize = 10;

pub type ReplInputChannel =
    channel::Channel<CriticalSectionRawMutex, ReplFlowControl<String>, REPL_IN_MTU>;
pub type ReplInputSender =
    channel::Sender<'static, CriticalSectionRawMutex, ReplFlowControl<String>, REPL_IN_MTU>;
pub type ReplInputReceiver =
    channel::Receiver<'static, CriticalSectionRawMutex, ReplFlowControl<String>, REPL_IN_MTU>;

pub type ReplOutputChannel =
    channel::Channel<CriticalSectionRawMutex, ReplPrintControl<String>, REPL_OUT_MTU>;
pub type ReplOutputSender =
    channel::Sender<'static, CriticalSectionRawMutex, ReplPrintControl<String>, REPL_OUT_MTU>;
pub type ReplOutputReceiver =
    channel::Receiver<'static, CriticalSectionRawMutex, ReplPrintControl<String>, REPL_OUT_MTU>;

// TODO: Need a Continue/Wait variant for ReplOut.
#[macro_export]
macro_rules! make_repl_channels {
    () => {{
        use crate::apps::rhai_repl::{ReplInputChannel, ReplOutputChannel};
        use embassy_sync::lazy_lock::LazyLock;

        static REPL_IN: LazyLock<ReplInputChannel> = LazyLock::new(|| ReplInputChannel::new());
        static REPL_OUT: LazyLock<ReplOutputChannel> = LazyLock::new(|| ReplOutputChannel::new());

        (REPL_IN.get(), REPL_OUT.get())
    }};
}

pub async fn repl_task(
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    in_rx: ReplInputReceiver,
    out_tx: ReplOutputSender,
    mut engine: Engine,
) -> ! {
    engine
        .on_print(move |s| {
            // NOTE: This can get clogged b/c the channel is too small.
            let _ = out_tx.try_send(ReplPrintControl::Continue(format!("{s}\n")));
        })
        .on_debug(move |s, _f, l| {
            // NOTE: This can get clogged b/c the channel is too small.
            let now = Instant::now();
            let _ = out_tx.try_send(ReplPrintControl::Debug(format!(
                "[{}s @ {}:{}] {}\n",
                now.as_secs(),
                l.line().unwrap_or(0),
                l.position().unwrap_or(0),
                s
            )));
            ()
        });

    #[allow(deprecated)]
    engine.register_debugger(
        // Provide a callback to initialize the debugger state
        |_engine, debugger| {
            // debugger.set_state(...);
            debugger
        },
        // Provide a callback for each debugging step
        |_context, _event, _node, _source, pos| {
            let sio = embassy_rp::pac::SIO;
            let status = sio.doorbell_in_clr().read();

            if status.doorbell_in_clr() != 0 {
                debug!("Received doorbell: {:?}", status.doorbell_in_clr());
                // Clear the interrupt
                sio.doorbell_in_clr().write_value(status);
                Err(EvalAltResult::ErrorTerminated("Ctrl-C".into(), pos).into())
            } else {
                Ok(DebuggerCommand::StepInto)
            }
        },
    );
    let mut scope = Scope::new();
    let mut ast = AST::empty();

    loop {
        debug!(
            "REPL HEAP: USED {:?} FREE {:?}",
            crate::HEAP.used(),
            crate::HEAP.free(),
        );

        match in_rx.receive().await {
            ReplFlowControl::Input(input) => {
                match engine.compile_with_scope(&scope, input.as_str()) {
                    Ok(local_ast) => {
                        debug!("AST: {:?}", defmt::Debug2Format(&local_ast));
                        debug!("SCOPE: {:?}", defmt::Debug2Format(&scope));
                        let new_ast = ast.merge(&local_ast);

                        match engine.eval_ast_with_scope::<Dynamic>(&mut scope, &new_ast) {
                            Ok(result) => {
                                debug!("Finished evaluating AST!");
                                out_tx
                                    .send(ReplPrintControl::Done(format!("{:?}", result)))
                                    .await;
                                ast = new_ast.clone_functions_only();
                                // ast =
                                //     engine.optimize_ast(&scope, ast, rhai::OptimizationLevel::Full);
                            }
                            Err(err) => {
                                out_tx
                                    .send(ReplPrintControl::Done(eval_result_to_str(&input, *err)))
                                    .await;
                            }
                        }
                    }
                    Err(err) => {
                        out_tx
                            .send(ReplPrintControl::Done(eval_result_to_str(
                                &input,
                                err.into(),
                            )))
                            .await;
                    }
                }
            }
            ReplFlowControl::Break => {
                warn!("Breaking REPL control flow!");
                let call = RpcCall::SysReleaseControl;
                let result = rpc_call_async_no_ctx(call_tx, result_rx, call).await;
                scope = Scope::new();
                warn!(
                    "Broken REPL control flow; control released: {:?}",
                    defmt::Debug2Format(&result)
                );
            }
        }
    }
}

fn eval_result_to_str(input: &str, mut err: EvalAltResult) -> String {
    let mut output = String::new();
    // Do not use `line` because it "eats" the last empty line if the script ends with a newline.
    let lines: alloc::vec::Vec<_> = input.split('\n').collect();
    let pos = err.take_position();

    let line_no = if lines.len() > 1 {
        if pos.is_none() {
            String::new()
        } else {
            format!("{}: ", pos.line().unwrap_or(0))
        }
    } else {
        String::new()
    };

    if pos.is_none() {
        // No position
        output.push_str(&format!("=> {err}\n"));
    } else {
        // Specific position - print line text
        output.push_str(&format!("{line_no}{}", lines[pos.line().unwrap_or(0) - 1]));

        let err_string = err.to_string();
        let err_enum = err_string.lines().enumerate();
        let err_len = err_enum.try_len().unwrap_or(1);

        for (i, err_line) in err_enum {
            if i == 0 {
                output.push_str("\n");
            }

            // Display position marker
            output.push_str(&format!(
                "   {0:>1$}{err_line}",
                if i > 0 { "| " } else { "^ " },
                line_no.len() + pos.position().unwrap_or(0) + 1,
            ));

            if i < err_len - 1 {
                output.push_str("\n");
            }
        }
    }

    output
}
