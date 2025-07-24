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
use static_cell::ConstStaticCell;

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

fn init_pwm<'scope>(
    pwm: Peri<'scope, peripherals::PWM0>,
    ch0: Peri<'scope, peripherals::P0_05>,
) -> SimplePwm<'scope, peripherals::PWM0> {
    let pwm_ctrl = SimplePwm::new_1ch(pwm, ch0);
    pwm_ctrl.set_prescaler(pwm::Prescaler::Div1);
    pwm_ctrl.set_period(2_000_000);

    pwm_ctrl
}

fn init_saadc<'scope>(
    saadc: Peri<'scope, peripherals::SAADC>,
    light_pin: Peri<'scope, peripherals::P0_02>,
    soil_pin: Peri<'scope, peripherals::P0_03>,
) -> Saadc<'scope, 3> {
    let light_config = ChannelConfig::single_ended(light_pin);

    let mut soil_config = ChannelConfig::single_ended(soil_pin);
    soil_config.reference = saadc::Reference::VDD1_4;

    let bat_config = ChannelConfig::single_ended(saadc::VddInput);

    let mut saadc_config = Config::default();
    saadc_config.resolution = Resolution::_10BIT;

    Saadc::new(
        saadc,
        Irqs,
        saadc_config,
        [soil_config, light_config, bat_config],
    )
}

#[embassy_executor::task]
pub async fn task(
    mut saadc: Peri<'static, peripherals::SAADC>,
    mut light_pin: Peri<'static, peripherals::P0_02>,
    mut soil_pin: Peri<'static, peripherals::P0_03>,
    mut photo_ctrl: Output<'static>,
    mut pwm: Peri<'static, peripherals::PWM0>,
    mut pin5: Peri<'static, peripherals::P0_05>,
) {
    static ADC_BUFFER: ConstStaticCell<[i16; 3]> = ConstStaticCell::new([0; 3]);
    let adc_buf = ADC_BUFFER.take();

    let mut measure = unwrap!(START_MEASUREMENTS.receiver());

    loop {
        measure.changed().await;

        let mut pwm_ctrl = init_pwm(pwm.reborrow(), pin5.reborrow());

        let mut saadc = init_saadc(saadc.reborrow(), light_pin.reborrow(), soil_pin.reborrow());

        photo_ctrl.set_high();
        pwm_ctrl.enable();
        pwm_ctrl.set_duty(0, 4);

        Timer::after_millis(30).await;

        let mut acc_buf = [0; 3];
        let divisor = 4;

        for _ in 0..divisor {
            saadc.sample(adc_buf).await;
            acc_buf
                .iter_mut()
                .zip(adc_buf.iter())
                .for_each(|(slot, &value)| *slot += value);
            Timer::after_millis(5).await;
        }

        photo_ctrl.set_low();
        pwm_ctrl.set_duty(0, 0);

        acc_buf.iter_mut().for_each(|acc| *acc /= divisor);

        let [soil, light, bat] = acc_buf;

        let bat_volt = to_volts(bat, VREF);

        let (soil, light, bat) = (
            calculate_soil_moisture(bat_volt, soil),
            calculate_lux(to_volts(light, VREF)).max(0.0),
            BatteryDischargeProfile::calc_pct_from_profile_range(
                bat_volt,
                DISCARGE_PROFILES.iter(),
            ),
        );

        let measurements = AdcMeasurements::new(bat, bat_volt, soil, light);

        info!("Soil {}, Light {}, Bat {}", soil, light, bat);

        ADC_MEASUREMENT.signal(measurements);
        pwm_ctrl.disable();
        drop(pwm_ctrl);
        drop(saadc);
    }
}
