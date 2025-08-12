use crate::{
    apps::rhai_repl::{ReplInputReceiver, ReplOutputSender},
    platform::repl::rpc::{RpcCallSender, RpcResultReceiver},
};
use defmt_rtt as _;
use embassy_time::Timer;
use panic_probe as _;
#[cfg(feature = "heap-in-psram")]
use platform::psram;

#[embassy_executor::task]
pub async fn repl_task(
    call_tx: RpcCallSender,
    result_rx: RpcResultReceiver,
    in_rx: ReplInputReceiver,
    out_tx: ReplOutputSender,
) {
    let engine = crate::platform::repl::make_engine(call_tx.clone(), result_rx.clone());
    crate::apps::rhai_repl::repl_task(call_tx, result_rx, in_rx, out_tx, engine).await;

    loop {
        Timer::after_secs(10).await
    }
}
