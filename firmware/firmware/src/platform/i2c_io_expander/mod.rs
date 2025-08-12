pub mod inner_pin;
pub mod model;
pub mod models;
pub mod pin;
pub mod pin_state;

use crate::platform::{
    i2c_io_expander::{inner_pin::InnerPin, model::ExpanderModel, pin::Pin},
    interrupt_i2c::update_state::UpdateState,
};
use alloc::sync::Arc;
use core::{array, marker::PhantomData};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_hal_async::i2c::I2c;
use itertools::Itertools;

pub struct Control<MUTEX, DEV, MODEL, const N: usize>
where
    MUTEX: RawMutex,
    DEV: I2c,
    MODEL: ExpanderModel<Inputs = [bool; N]>,
{
    dev: DEV,
    inner_pins: [Arc<InnerPin<MUTEX>>; N],
    _null0: PhantomData<MODEL>,
}

impl<MUTEX, DEV, MODEL, const N: usize> Control<MUTEX, DEV, MODEL, N>
where
    MUTEX: RawMutex,
    DEV: I2c,
    MODEL: ExpanderModel<Inputs = [bool; N]>,
{
    pub fn new(dev: DEV, inner_pins: [Arc<InnerPin<MUTEX>>; N]) -> Self {
        Self {
            dev,
            inner_pins: inner_pins,
            _null0: PhantomData::default(),
        }
    }

    pub async fn read_inputs(&mut self) -> MODEL::Inputs {
        let inputs = MODEL::read_inputs(&mut self.dev).await;

        for (i, pin) in self.inner_pins.iter().enumerate() {
            pin.notify_input(inputs[i]).await;
        }

        inputs
    }
}

impl<MUTEX, DEV, MODEL, const N: usize> UpdateState for Control<MUTEX, DEV, MODEL, N>
where
    MUTEX: RawMutex,
    DEV: I2c,
    MODEL: ExpanderModel<Inputs = [bool; N]>,
{
    async fn update(&mut self) {
        self.read_inputs().await;
    }
}

pub fn setup<MUTEX, DEV, MODEL, const N: usize>(
    bus: &'static Mutex<MUTEX, DEV>,
) -> (
    Control<MUTEX, I2cDevice<'static, MUTEX, DEV>, MODEL, N>,
    Control<MUTEX, I2cDevice<'static, MUTEX, DEV>, MODEL, N>,
    [Pin<MUTEX, I2cDevice<'static, MUTEX, DEV>, MODEL>; N],
)
where
    MUTEX: RawMutex,
    DEV: I2c,
    MODEL: ExpanderModel<Inputs = [bool; N]>,
{
    let pins: [Pin<MUTEX, _, MODEL>; N] =
        array::from_fn(|id| Pin::new(I2cDevice::new(bus), id as u8));
    let inner_pins = pins
        .iter()
        .map(|pin| pin.get_inner_pin())
        .collect_array()
        .unwrap();

    (
        Control::new(I2cDevice::new(bus), inner_pins.clone()),
        Control::new(I2cDevice::new(bus), inner_pins),
        pins,
    )
}
