#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{clock::ClockControl, Rng, embassy, peripherals::Peripherals, prelude::*, systimer, timer::TimerGroup};
use esp_println::println;

#[main]
async fn main(spawner: Spawner) {
    println!("Init!");
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    spawner.spawn(run()).ok();

    loop {
        println!("Bing!");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

#[embassy_executor::task]
async fn run() {
    loop {
        println!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}
