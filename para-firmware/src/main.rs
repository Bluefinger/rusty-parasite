#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

mod adc;
mod fmt;
mod shtc3;

#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts, gpio::{Level, Output, OutputDrive}, peripherals, pwm::SimplePwm, saadc, twim
};
use embassy_time::Timer;
use fmt::info;

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
    SAADC => saadc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_28, Level::Low, OutputDrive::Standard);
    let photo_ctrl = Output::new(p.P0_29, Level::Low, OutputDrive::Standard);
    let pwm_ctrl = SimplePwm::new_1ch(p.PWM0, p.P0_05);

    spawner.must_spawn(shtc3::task(p.TWISPI0, p.P0_24, p.P0_13));
    spawner.must_spawn(adc::task(p.SAADC, p.P0_02, p.P0_03, photo_ctrl, pwm_ctrl));

    info!("Rusty Parasite is go!");

    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}
