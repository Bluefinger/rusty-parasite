use embassy_nrf::{
    gpio::Output,
    peripherals,
    pwm::{self, SimplePwm},
    saadc::{self, ChannelConfig, Config, Resolution, Saadc},
};
use embassy_time::{Duration, Instant, Timer};
use para_battery::BatteryDischargeProfile;

use crate::{Irqs, info};

static DRY_COEFFS: [f32; 3] = [334.0, 110.0, -15.3];
static WET_COEFFS: [f32; 3] = [299.0, -83.1, 11.2];
static DISCARGE_PROFILES: [BatteryDischargeProfile; 4] = [
    BatteryDischargeProfile::new(3.00, 2.90, 1.00, 0.42),
    BatteryDischargeProfile::new(2.90, 2.74, 0.42, 0.18),
    BatteryDischargeProfile::new(2.74, 2.44, 0.18, 0.06),
    BatteryDischargeProfile::new(2.44, 2.01, 0.06, 0.00),
];

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

    ((soil as f32) - dry) / (wet - dry)
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
    saadc: peripherals::SAADC,
    light_pin: peripherals::P0_02,
    soil_pin: peripherals::P0_03,
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

    loop {
        let now = Instant::now();

        photo_ctrl.set_high();
        pwm_ctrl.set_duty(0, 4);

        Timer::after_millis(30).await;

        saadc.sample(&mut buf).await;

        let [soil, light, bat] = buf;

        let bat_volt = to_volts(bat, VREF);

        info!(
            "ADC readings: soil {}%, lux {}, bat {}v {}%",
            calculate_soil_moisture(bat_volt, soil) * 100.0,
            calculate_lux(to_volts(light, VREF)),
            bat_volt,
            BatteryDischargeProfile::calc_pct_from_profile_range(
                bat_volt,
                DISCARGE_PROFILES.iter()
            ) * 100.0
        );

        photo_ctrl.set_low();
        pwm_ctrl.set_duty(0, 0);

        let delay = Duration::from_secs(3).as_ticks() - now.elapsed().as_ticks();

        Timer::after_ticks(delay).await;
    }
}
