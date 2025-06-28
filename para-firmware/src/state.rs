use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal, watch::Watch};
use para_bthome::{
    Battery1Per, Humidity10mPer, Illuminance10mLux, Moisture10mPer, Temperature10mK,
};
use para_shtc3::Measurement;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AdcMeasurements {
    pub battery: Battery1Per,
    pub moisture: Moisture10mPer,
    pub lux: Illuminance10mLux,
}

impl AdcMeasurements {
    pub fn new(battery: f32, moisture: f32, lux: f32) -> Self {
        let battery = (battery * 100.0) as u8;
        let moisture = (moisture * 10000.0) as u16;
        let lux = lux as u32;

        Self {
            battery: battery.into(),
            moisture: moisture.into(),
            lux: lux.into(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Shtc3Measurement {
    pub temperature: Temperature10mK,
    pub humidity: Humidity10mPer,
}

impl Shtc3Measurement {
    pub fn new(measurement: Measurement) -> Self {
        Self {
            temperature: measurement.temperature.as_10mk_celsius().into(),
            humidity: measurement.humidity.as_10mk_percent().into(),
        }
    }
}

pub static SHTC3_MEASUREMENT: Signal<ThreadModeRawMutex, Shtc3Measurement> = Signal::new();
pub static ADC_MEASUREMENT: Signal<ThreadModeRawMutex, AdcMeasurements> = Signal::new();
pub static START_MEASUREMENTS: Watch<ThreadModeRawMutex, (), 3> = Watch::new();
