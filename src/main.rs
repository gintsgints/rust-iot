#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::{task, Spawner};
use embassy_net::{Config, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, embassy, peripherals::Peripherals, prelude::*, systimer::SystemTimer,
    timer::TimerGroup, Rng,
};
use esp_println::println;
use esp_wifi::{
    initialize,
    wifi::{
        AccessPointInfo, ClientConfiguration, Configuration, WifiController, WifiDevice, WifiError, WifiEvent, WifiStaDevice, WifiState
    },
    EspWifiInitFor,
};

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[main]
async fn main(spawner: Spawner) {
    println!("Init!");
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    let wifi = peripherals.WIFI;
    let (_wifi_interface, mut controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    let client_config = Configuration::Client(ClientConfiguration {
        // ANCHOR_END: client_config_start
        ssid: SSID.try_into().unwrap(),
        // if I put wrong password here connection fails
        password: PASSWORD.try_into().unwrap(),
        // and if I uncoment next line, connection fails
        auth_method: esp_wifi::wifi::AuthMethod::None,
        ..Default::default() // ANCHOR: client_config_end
    });

    let res = controller.set_configuration(&client_config);
    println!("Wi-Fi set_configuration returned {:?}", res);

    controller.start().await.unwrap();
    println!("Is wifi started: {:?}", controller.is_started());

    println!("Start Wifi Scan");
    let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> = controller.scan_n().await;
    if let Ok((res, _count)) = res {
        for ap in res {
            println!("{:?}", ap);
        }
    }

    println!("{:?}", controller.get_capabilities());
    println!("Wi-Fi connect: {:?}", controller.connect().await);
    
    // Wait to get connected
    println!("Wait to get connected");
    loop {
        let res = controller.is_connected();
        match res {
            Ok(connected) => {
                if connected {
                    break;
                }
            }
            Err(err) => {
                println!("{:?}", err);
                loop {}
            }
        }
    }
    
    spawner.spawn(run()).ok();
    // spawner.spawn(connection(controller)).ok();
    // spawner.spawn(net_task(&stack)).ok();

    loop {
        println!("Bing!");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

#[task]
async fn run() {
    loop {
        println!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

// #[task]
// async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
//     stack.run().await
// }

// #[task]
// async fn connection(mut controller: WifiController<'static>) {
//     println!("start connection task");
//     println!("Device capabilities: {:?}", controller.get_capabilities());
//     loop {
//         match esp_wifi::wifi::get_wifi_state() {
//             WifiState::StaConnected => {
//                 // wait until we're no longer connected
//                 controller.wait_for_event(WifiEvent::StaDisconnected).await;
//                 Timer::after(Duration::from_millis(5000)).await
//             }
//             _ => {}
//         }
//         if !matches!(controller.is_started(), Ok(true)) {
//             let client_config = Configuration::Client(ClientConfiguration {
//                 ssid: SSID.try_into().unwrap(),
//                 password: PASSWORD.try_into().unwrap(),
//                 ..Default::default()
//             });
//             controller.set_configuration(&client_config).unwrap();
//             println!("Starting wifi");
//             controller.start().await.unwrap();
//             println!("Wifi started!");
//         }
//         println!("About to connect...");

//         match controller.connect().await {
//             Ok(_) => println!("Wifi connected!"),
//             Err(e) => {
//                 println!("Failed to connect to wifi: {e:?}");
//                 Timer::after(Duration::from_millis(5000)).await
//             }
//         }
//     }
// }
