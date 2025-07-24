use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal, watch::Watch};
use para_bthome::{Battery1Per, Humidity1Per, Illuminance10mLux, Moisture1Per, Temperature10mK, Voltage1mV};
use para_shtc3::Measurement;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(align(8))]
pub struct AdcMeasurements {
    pub battery: Battery1Per,
    pub voltage: Voltage1mV,
    pub moisture: Moisture1Per,
    pub lux: Illuminance10mLux,
}

impl AdcMeasurements {
    pub fn new(battery: f32, voltage: f32, moisture: f32, lux: f32) -> Self {
        let battery = (battery * 100.0) as u8;
        let voltage = (voltage * 1000.0) as u16;
        let moisture = (moisture * 100.0) as u8;
        let lux = (lux * 100.0) as u32;

        Self {
            battery: battery.into(),
            voltage: voltage.into(),
            moisture: moisture.into(),
            lux: lux.into(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(align(8))]
pub struct Shtc3Measurement {
    pub temperature: Temperature10mK,
    pub humidity: Humidity1Per,
}

impl Shtc3Measurement {
    pub fn new(measurement: Measurement) -> Self {
        Self {
            temperature: measurement.temperature.as_10mk_celsius().into(),
            humidity: measurement.humidity.as_1k_percent().into(),
        }
    }
}

pub static SHTC3_MEASUREMENT: Signal<ThreadModeRawMutex, Shtc3Measurement> = Signal::new();
pub static ADC_MEASUREMENT: Signal<ThreadModeRawMutex, AdcMeasurements> = Signal::new();
pub static START_MEASUREMENTS: Watch<ThreadModeRawMutex, (), 4> = Watch::new();
