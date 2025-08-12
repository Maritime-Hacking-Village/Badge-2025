use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex, signal::Signal};

use crate::platform::i2c_io_expander::pin_state::PinState;

pub struct InnerPin<M: RawMutex> {
    pub state: Mutex<M, PinState>,
    signal: Signal<M, bool>,
}

impl<M: RawMutex> InnerPin<M> {
    pub fn new() -> Self {
        let state = Mutex::new(PinState {
            input: true,
            output: true,
            direction: true,
        });
        let signal = Signal::new();
        Self { state, signal }
    }

    pub async fn wait(&self) -> bool {
        self.signal.wait().await
    }
}

impl<M: RawMutex> InnerPin<M> {
    pub async fn notify_input(&self, input: bool) {
        // only if it changed
        let mut state = self.state.lock().await;
        if state.input != input {
            // First set the input then notify
            state.input = input;
            self.signal.signal(input);
        }
    }
}
