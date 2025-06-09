use embassy_nrf::{
    peripherals,
    twim::{self, Twim},
};
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::i2c::SevenBitAddress;
use para_shtc3::{Error as ShtError, PowerMode, ShtC3};

use crate::{Irqs, fmt::error, info};

async fn measure<I>(sht: &mut ShtC3<I>) -> Result<(), ShtError<I::Error>>
where
    I: embedded_hal::i2c::I2c<SevenBitAddress>,
{
    sht.start_wakeup()?;

    Timer::after_micros(sht.wakeup_duration() as u64).await;

    let mode = PowerMode::NormalMode;

    sht.start_measurement(mode)?;

    Timer::after_micros(sht.max_measurement_duration(mode) as u64).await;

    let m = sht.get_measurement_result()?;

    info!(
        "T: {}C, H: {}%",
        m.temperature.as_degrees_celsius(),
        m.humidity.as_percent()
    );

    sht.sleep()?;

    Ok(())
}

async fn reset<I>(sht: &mut ShtC3<I>) -> Result<(), ShtError<I::Error>>
where
    I: embedded_hal::i2c::I2c<SevenBitAddress>,
{
    sht.start_reset()?;

    Timer::after_micros(sht.reset_duration() as u64).await;

    Ok(())
}

#[embassy_executor::task]
pub async fn task(spio: peripherals::TWISPI0, sda: peripherals::P0_24, scl: peripherals::P0_13) {
    let config = twim::Config::default();

    let twi = Twim::new(spio, Irqs, sda, scl, config);

    let mut sht = ShtC3::new(twi);

    loop {
        let now = Instant::now();

        if let Err(e) = measure(&mut sht).await {
            error!("SHTC3 error: {:?}", e);

            // Attempt to reset the sensor
            if let Err(e) = reset(&mut sht).await {
                error!("SHTC3 reset error: {:?}", e);
            }
        }

        let delay = Duration::from_secs(3).as_ticks() - now.elapsed().as_ticks();

        Timer::after_ticks(delay).await;
    }
}
