#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

mod adc;
mod ble;
mod button;
mod constants;
mod led;
mod shtc3;
mod state;
mod timer;

#[cfg(not(feature = "defmt"))]
use panic_halt as _;
use para_fmt::{info, unwrap};
use static_cell::StaticCell;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive},
    peripherals,
    rng, saadc, twim,
};
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, mpsl};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
    EGU0_SWI0 => nrf_sdc::mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => nrf_sdc::mpsl::ClockInterruptHandler;
    RADIO => nrf_sdc::mpsl::HighPrioInterruptHandler;
    TIMER0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    RTC0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
    SAADC => saadc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    spawner.must_spawn(button::task(Input::new(
        p.P0_30,
        embassy_nrf::gpio::Pull::Up,
    )));
    spawner.must_spawn(led::task(Output::new(
        p.P0_28,
        Level::Low,
        OutputDrive::Standard,
    )));

    let photo_ctrl = Output::new(p.P0_29, Level::Low, OutputDrive::Standard);

    spawner.must_spawn(shtc3::task(p.TWISPI0, p.P0_24, p.P0_13));
    spawner.must_spawn(adc::task(
        p.SAADC, p.P0_02, p.P0_03, photo_ctrl, p.PWM0, p.P0_05,
    ));
    spawner.must_spawn(timer::task());

    let mpsl_p =
        mpsl::Peripherals::new(p.RTC0, p.TIMER0, p.TEMP, p.PPI_CH19, p.PPI_CH30, p.PPI_CH31);
    let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: mpsl::raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
        skip_wait_lfclk_started: mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };
    static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
    let mpsl = MPSL.init(unwrap!(mpsl::MultiprotocolServiceLayer::new(
        mpsl_p, Irqs, lfclk_cfg
    )));
    spawner.must_spawn(ble::mpsl_task(&*mpsl));

    let sdc_p = sdc::Peripherals::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
        p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
    );

    let mut rng = rng::Rng::new(p.RNG, Irqs);

    let mut sdc_mem = sdc::Mem::<1648>::new();
    let sdc = unwrap!(ble::build_sdc(sdc_p, &mut rng, mpsl, &mut sdc_mem));

    info!("Rusty Parasite is go!");

    ble::run(sdc).await;
}
