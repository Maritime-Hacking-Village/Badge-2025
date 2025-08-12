use crate::platform::i2c_io_expander::{inner_pin::InnerPin, model::ExpanderModel};
use alloc::sync::Arc;
use core::marker::PhantomData;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embedded_hal_async::{
    digital::{self, InputPin, OutputPin, StatefulOutputPin, Wait},
    i2c,
    i2c::I2c,
};

pub struct Pin<M: RawMutex, D: I2c + i2c::ErrorType, MODEL: ExpanderModel> {
    id: u8,
    inner_pin: Arc<InnerPin<M>>,
    dev: D,
    _null0: PhantomData<MODEL>,
}

impl<M: RawMutex, D: I2c + i2c::ErrorType, MODEL: ExpanderModel> Pin<M, D, MODEL> {
    pub fn new(dev: D, id: u8) -> Self {
        let inner_pin = Arc::new(InnerPin::new());
        Pin {
            id,
            inner_pin,
            dev,
            _null0: PhantomData,
        }
    }
    pub fn get_inner_pin(&self) -> Arc<InnerPin<M>> {
        self.inner_pin.clone()
    }

    pub async fn set_direction(&mut self, direction: bool) {
        let mut state = self.inner_pin.state.lock().await;
        MODEL::set_direction(&mut self.dev, self.id, direction).await;
        state.direction = direction;
    }

    pub async fn get_direction(&self) -> bool {
        self.inner_pin.state.lock().await.direction
    }

    pub async fn set_output(&mut self, output: bool) {
        let mut state = self.inner_pin.state.lock().await;
        MODEL::set_output(&mut self.dev, self.id, output).await;
        state.output = output;
    }

    pub async fn notify_input(&self, input: bool) {
        self.inner_pin.notify_input(input).await;
    }
}

impl<M: RawMutex, D: I2c, MODEL: ExpanderModel> digital::ErrorType for Pin<M, D, MODEL> {
    type Error = digital::ErrorKind;
}

impl<M: RawMutex, D: I2c, MODEL: ExpanderModel> OutputPin for Pin<M, D, MODEL> {
    async fn set_low(&mut self) -> Result<(), Self::Error> {
        const STATE: bool = false;
        self.set_output(STATE).await;

        if self.inner_pin.state.lock().await.input {
            self.notify_input(STATE).await;
        }
        Ok(())
    }

    async fn set_high(&mut self) -> Result<(), Self::Error> {
        const STATE: bool = true;
        self.set_output(STATE).await;
        if !self.inner_pin.state.lock().await.input {
            self.notify_input(STATE).await;
        }
        Ok(())
    }
}

impl<M: RawMutex, D: I2c, MODEL: ExpanderModel> StatefulOutputPin for Pin<M, D, MODEL> {
    async fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.inner_pin.state.lock().await.output)
    }

    async fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.inner_pin.state.lock().await.output)
    }
}

impl<M: RawMutex, D: I2c, MODEL: ExpanderModel> InputPin for Pin<M, D, MODEL> {
    async fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.inner_pin.state.lock().await.input)
    }

    async fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.inner_pin.state.lock().await.input)
    }
}

impl<M: RawMutex, D: I2c, MODEL: ExpanderModel> Wait for Pin<M, D, MODEL> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        while !self.inner_pin.state.lock().await.input {
            self.inner_pin.wait().await;
        }
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        while self.inner_pin.state.lock().await.input {
            self.inner_pin.wait().await;
        }
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        loop {
            self.inner_pin.wait().await;
            if self.inner_pin.state.lock().await.input {
                break;
            };
        }
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        loop {
            self.inner_pin.wait().await;
            if !self.inner_pin.state.lock().await.input {
                break;
            };
        }
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.inner_pin.wait().await;
        Ok(())
    }
}
