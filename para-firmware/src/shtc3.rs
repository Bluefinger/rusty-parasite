use embassy_nrf::{
    Peri, peripherals,
    twim::{self, Twim},
};
use embassy_time::Timer;
use embedded_hal::i2c::SevenBitAddress;
use para_fmt::{error, unwrap};
use para_shtc3::{Error as ShtError, Measurement, PowerMode, ShtC3};
use static_cell::ConstStaticCell;

use crate::{
    Irqs, info,
    state::{SHTC3_MEASUREMENT, START_MEASUREMENTS, Shtc3Measurement},
};

async fn measure<I>(sht: &mut ShtC3<I>) -> Result<Measurement, ShtError<I::Error>>
where
    I: embedded_hal::i2c::I2c<SevenBitAddress>,
{
    sht.start_wakeup()?;

    Timer::after_micros(sht.wakeup_duration() as u64).await;

    let mode = PowerMode::LowPower;

    let divisor = 4;
    let mut m = Measurement::default();

    for _ in 0..divisor {
        sht.start_measurement(mode)?;

        Timer::after_micros(sht.max_measurement_duration(mode) as u64).await;

        m += sht.get_measurement_result()?;

        Timer::after_millis(5).await;
    }

    m /= divisor;

    info!(
        "Temp: {}C, Humi: {}%",
        m.temperature.as_degrees_celsius(),
        m.humidity.as_percent()
    );

    sht.sleep()?;

    Ok(m)
}

async fn reset<I>(sht: &mut ShtC3<I>) -> Result<(), ShtError<I::Error>>
where
    I: embedded_hal::i2c::I2c<SevenBitAddress>,
{
    sht.start_reset()?;

    Timer::after_micros(sht.reset_duration() as u64).await;

    Ok(())
}

fn init_sht3<'scope>(
    spio: Peri<'scope, peripherals::TWISPI0>,
    sda: Peri<'scope, peripherals::P0_24>,
    scl: Peri<'scope, peripherals::P0_13>,
    ram: &'scope mut [u8; 16],
) -> ShtC3<Twim<'scope, peripherals::TWISPI0>> {
    let config = twim::Config::default();
    let twi = Twim::new(spio, Irqs, sda, scl, config, ram);

    ShtC3::new(twi)
}

#[embassy_executor::task]
pub async fn task(
    mut spio: Peri<'static, peripherals::TWISPI0>,
    mut sda: Peri<'static, peripherals::P0_24>,
    mut scl: Peri<'static, peripherals::P0_13>,
) {
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let ram = RAM_BUFFER.take();

    let mut watcher = unwrap!(START_MEASUREMENTS.receiver());

    loop {
        watcher.changed().await;

        let mut sht = init_sht3(spio.reborrow(), sda.reborrow(), scl.reborrow(), ram);

        match measure(&mut sht).await {
            Ok(measurement) => {
                SHTC3_MEASUREMENT.signal(Shtc3Measurement::new(measurement));
            }
            Err(e) => {
                error!("SHTC3 error: {:?}", e);

                // Attempt to reset the sensor
                if let Err(e) = reset(&mut sht).await {
                    error!("SHTC3 reset error: {:?}", e);
                }
            }
        }

        drop(sht);
    }
}
