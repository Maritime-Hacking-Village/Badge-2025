pub mod update_state;

use defmt::warn;
use embassy_time::Instant;
use embedded_hal_async::digital::Wait;
use update_state::UpdateState;

pub fn setup<P: Wait, U1: UpdateState, U2: UpdateState, U3: UpdateState>(
    pin: P,
    gpio: U1,
    batt: U2,
    accel: U3,
) -> Runner<P, U1, U2, U3> {
    Runner {
        pin,
        gpio,
        batt,
        accel,
    }
}

pub struct Runner<P, U1, U2, U3>
where
    P: Wait,
    U1: UpdateState,
    U2: UpdateState,
    U3: UpdateState,
{
    pin: P,
    gpio: U1,
    batt: U2,
    accel: U3,
}

const LOG_FLAG_LEN: usize = 22;
const LOG_FLAG: [u8; LOG_FLAG_LEN] = [
    0x93, 0xE5, 0x96, 0xA5, 0x8D, 0xC8, 0xB2, 0xB7, 0xBF, 0xC1, 0x93, 0xAA, 0xBB, 0xCA, 0xAF, 0x8D,
    0xB0, 0xC4, 0xB0, 0xBB, 0xAC, 0xDE,
];

impl<P, U1, U2, U3> Runner<P, U1, U2, U3>
where
    P: Wait,
    U1: UpdateState,
    U2: UpdateState,
    U3: UpdateState,
{
    pub async fn run(&mut self) -> ! {
        let mut i = 0;

        loop {
            let t0 = Instant::now();
            self.gpio.update().await;
            let t1 = Instant::now();
            self.batt.update().await;
            let t2 = Instant::now();
            self.accel.update().await;
            let t3 = Instant::now();

            warn!("{:02X}", LOG_FLAG[i]);
            i += 1;

            if i >= LOG_FLAG_LEN {
                i = 0;
            }

            self.pin.wait_for_low().await.unwrap();
        }
    }
}
