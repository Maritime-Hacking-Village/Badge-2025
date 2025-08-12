use crate::platform::repl::rpc::{RpcError, RpcResult};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel, signal};

pub type AckSignal = signal::Signal<CriticalSectionRawMutex, Result<RpcResult, RpcError>>;

#[derive(Debug, Clone)]
pub enum ControlCommand {
    SetDisplayBacklight(u8),
    ResetDisplay,
    GetTerm,
    SetTerm(bool, bool),
    GetTie,
    SetTie(bool),
    GetSaoGpioDir,
    SetSaoGpioDir(bool, bool),
    WriteSaoGpio(bool, bool),
    ReadSaoGpio,
}

pub const CTRL_MTU: usize = 1;

pub type ControlChannel = channel::Channel<CriticalSectionRawMutex, ControlCommand, CTRL_MTU>;
pub type ControlSender =
    channel::Sender<'static, CriticalSectionRawMutex, ControlCommand, CTRL_MTU>;
pub type ControlReceiver =
    channel::Receiver<'static, CriticalSectionRawMutex, ControlCommand, CTRL_MTU>;

#[macro_export]
macro_rules! make_ctrl_channel {
    () => {{
        use crate::platform::repl::common::{AckSignal, ControlChannel};
        use embassy_sync::lazy_lock::LazyLock;

        static CHANNEL: LazyLock<ControlChannel> = LazyLock::new(|| ControlChannel::new());
        static SIGNAL: LazyLock<AckSignal> = LazyLock::new(|| AckSignal::new());

        (CHANNEL.get(), SIGNAL.get())
    }};
}
