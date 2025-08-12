use cyw43::{Control, NetDriver};
use embassy_net::{Runner, Stack, StackResources};
use embassy_rp::clocks::RoscRng;
// use rand::RngCore;
use static_cell::StaticCell;

pub async fn setup(
    control: &mut Control<'static>,
    net_device: NetDriver<'static>,
) -> (Stack<'static>, Runner<'static, cyw43::NetDriver<'static>>) {
    let clm = include_bytes!("../../cyw43-firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download ../../embassy/cyw43-firmware/43439A0_clm.bin --binary-format bin --chip RP235X --base-address 0x10240000
    // let clm = unsafe { core::slice::from_raw_parts(0x10240000 as *const u8, 4752) };

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let config = embassy_net::Config::dhcpv4(Default::default());
    // let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //     address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 2), 24),
    //     dns_servers: Vec::new(),
    //     gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    // });

    // Generate random seed
    let mut rng = RoscRng;
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

    embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    )
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}
