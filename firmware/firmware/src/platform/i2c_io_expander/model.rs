use embedded_hal_async::i2c::I2c;

pub trait ExpanderModel {
    type Inputs;
    async fn set_direction<D: I2c>(dev: &mut D, id: u8, direction: bool);
    async fn set_output<D: I2c>(dev: &mut D, id: u8, output: bool);
    async fn read_inputs<D: I2c>(dev: &mut D) -> Self::Inputs;
}
