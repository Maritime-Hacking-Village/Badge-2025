use embedded_hal::digital::{ErrorType, InputPin as InputPinSync, OutputPin as OutputPinSync};
use embedded_hal_async::digital::{InputPin as InputPinAsync, OutputPin as OutputPinAsync};

pub struct AsyncOutputPin<T>(pub T)
where
    T: OutputPinSync + ErrorType;

impl<T> ErrorType for AsyncOutputPin<T>
where
    T: OutputPinSync + ErrorType,
{
    type Error = T::Error;
}

impl<T> OutputPinAsync for AsyncOutputPin<T>
where
    T: OutputPinSync + ErrorType,
{
    async fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low()
    }

    async fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high()
    }
}

pub struct AsyncInputPin<T>(pub T)
where
    T: InputPinSync + ErrorType;

impl<T> ErrorType for AsyncInputPin<T>
where
    T: InputPinSync + ErrorType,
{
    type Error = T::Error;
}

impl<T> InputPinAsync for AsyncInputPin<T>
where
    T: InputPinSync + ErrorType,
{
    async fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.0.is_high()
    }

    async fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.0.is_low()
    }
}
