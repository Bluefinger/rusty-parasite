#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

mod fmt;
mod shtc3;

#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    peripherals, twim,
};
use embassy_time::Timer;
use fmt::info;

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_28, Level::Low, OutputDrive::Standard);

    spawner.must_spawn(shtc3::task(p.TWISPI0, p.P0_24, p.P0_13));

    info!("Rusty Parasite is go!");

    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}
