use embassy_nrf::{
    Peri,
    gpio::Output,
    peripherals,
    pwm::{self, SimplePwm},
    saadc::{self, ChannelConfig, Config, Resolution, Saadc},
};
use embassy_time::Timer;
use para_battery::BatteryDischargeProfile;
use para_fmt::{info, unwrap};

use crate::{
    Irqs,
    constants::{DISCARGE_PROFILES, DRY_COEFFS, WET_COEFFS},
    state::{ADC_MEASUREMENT, AdcMeasurements, START_MEASUREMENTS},
};

const VREF: f32 = 3.6;

#[inline]
fn calculate_polynomial(coeffs: &[f32; 3], val: f32) -> f32 {
    coeffs[0] + (coeffs[1] * val) + (coeffs[2] * (val * val))
}

#[inline]
fn calculate_soil_moisture(bat: f32, soil: i16) -> f32 {
    let dry = calculate_polynomial(&DRY_COEFFS, bat);
    let wet = calculate_polynomial(&WET_COEFFS, bat);

    info!("WUH: dry {}, wet {}, soil {}", dry, wet, soil);

    (((soil as f32) - dry) / (wet - dry)).clamp(0.0, 1.0)
}

#[inline]
fn calculate_lux(voltage: f32) -> f32 {
    const LUX_SUN: f32 = 10000.0;
    const CURRENT_SUN: f32 = 3.59e-3;
    const PHOTO_RESISTOR: f32 = 470.0;

    let current = voltage / PHOTO_RESISTOR;

    LUX_SUN * current / CURRENT_SUN
}

#[inline]
fn to_volts(sample: i16, reference: f32) -> f32 {
    ((sample.max(0) as f32) * reference) / 1024.0
}

#[embassy_executor::task]
pub async fn task(
    saadc: Peri<'static, peripherals::SAADC>,
    light_pin: Peri<'static, peripherals::P0_02>,
    soil_pin: Peri<'static, peripherals::P0_03>,
    mut photo_ctrl: Output<'static>,
    mut pwm_ctrl: SimplePwm<'static, peripherals::PWM0>,
) {
    let light_config = ChannelConfig::single_ended(light_pin);
    let mut soil_config = ChannelConfig::single_ended(soil_pin);
    soil_config.reference = saadc::Reference::VDD1_4;
    let bat_config = ChannelConfig::single_ended(saadc::VddInput);

    let mut saadc_config = Config::default();

    saadc_config.resolution = Resolution::_10BIT;

    let mut saadc = Saadc::new(
        saadc,
        Irqs,
        saadc_config,
        [soil_config, light_config, bat_config],
    );

    let mut buf = [0; 3];

    saadc.calibrate().await;

    pwm_ctrl.set_prescaler(pwm::Prescaler::Div1);
    pwm_ctrl.set_period(2_000_000);

    info!("max duty {}", pwm_ctrl.max_duty());

    pwm_ctrl.enable();

    let mut measure = unwrap!(START_MEASUREMENTS.receiver());

    loop {
        measure.changed().await;

        photo_ctrl.set_high();
        pwm_ctrl.set_duty(0, 4);

        Timer::after_millis(30).await;

        saadc.sample(&mut buf).await;

        photo_ctrl.set_low();
        pwm_ctrl.set_duty(0, 0);

        let [soil, light, bat] = buf;

        let bat_volt = to_volts(bat, VREF);

        let measurements = AdcMeasurements::new(
            BatteryDischargeProfile::calc_pct_from_profile_range(
                bat_volt,
                DISCARGE_PROFILES.iter(),
            ),
            calculate_soil_moisture(bat_volt, soil),
            calculate_lux(to_volts(light, VREF)).max(0.0),
        );

        info!("{:?}", &measurements);

        ADC_MEASUREMENT.signal(measurements);
    }
}
