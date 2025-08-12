// use super::rhai_repl::handle_client;
// use crate::make_rhai_channel;
use cyw43::{Control, JoinOptions};
use defmt::{info, warn};
use embassy_net::{tcp::TcpSocket, Stack};
use embassy_time::{Duration, Timer};
// use rhai::Engine;

#[embassy_executor::task]
pub async fn cli_task(mut control: Control<'static>, stack: Stack<'static>) -> ! {
    const WIFI_NETWORK: &str = "TP-Link_A820";
    const WIFI_PASSWORD: &str = "Senha Secreta";
    loop {
        match control
            .join(WIFI_NETWORK, JoinOptions::new(WIFI_PASSWORD.as_bytes()))
            .await
        {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }

    // Wait for DHCP, not necessary when using static IP
    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("Network is now up!");

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    // let mut engine = Engine::new();

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        control.gpio_set(0, false).await;
        info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());
        control.gpio_set(0, true).await;

        // handle_client(&mut socket, make_rhai_channel!(), &mut engine)
        //     .await
        //     .unwrap_or_else(|err| warn!("client error: {:?}", err));
        socket.close();
    }
}
