use embassy_rp::spi::Config;

use crate::platform::set_frequency::SetFrequency;

impl SetFrequency for Config {
    fn set_frequency(&mut self, frequency: u32) {
        self.frequency = frequency
    }
}
