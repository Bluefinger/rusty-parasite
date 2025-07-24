/// A temperature measurement.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Temperature(i32);

/// A humidity measurement.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Humidity(i32);

/// A combined temperature / humidity measurement.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Measurement {
    /// The measured temperature.
    pub temperature: Temperature,
    /// The measured humidity.
    pub humidity: Humidity,
}

impl core::ops::AddAssign for Measurement {
    fn add_assign(&mut self, rhs: Self) {
        self.temperature.0 += rhs.temperature.0;
        self.humidity.0 += rhs.humidity.0;
    }
}

impl core::ops::DivAssign<i32> for Measurement {
    fn div_assign(&mut self, rhs: i32) {
        self.temperature.0 /= rhs;
        self.humidity.0 /= rhs;
    }
}

/// A combined raw temperature / humidity measurement.
///
/// The raw values are of type u16. They require a conversion formula for
/// conversion to a temperature / humidity value (see datasheet).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawMeasurement {
    /// The measured temperature (raw value).
    pub temperature: u16,
    /// The measured humidity (raw value).
    pub humidity: u16,
}

impl From<RawMeasurement> for Measurement {
    fn from(other: RawMeasurement) -> Self {
        Self {
            temperature: Temperature::from_raw(other.temperature),
            humidity: Humidity::from_raw(other.humidity),
        }
    }
}

impl Temperature {
    /// Create a new `Temperature` from a raw measurement result.
    pub const fn from_raw(raw: u16) -> Self {
        Self(convert_temperature(raw))
    }

    /// Return temperature in milli-degrees celsius.
    pub const fn as_millidegrees_celsius(&self) -> i32 {
        self.0
    }

    /// Return temperature in degrees celcius with 0.01 precision
    pub const fn as_10mk_celsius(&self) -> i16 {
        (self.0 / 10) as i16
    }

    /// Return temperature in degrees celsius.
    pub const fn as_degrees_celsius(&self) -> f32 {
        self.0 as f32 / 1000.0
    }
}

impl Humidity {
    /// Create a new `Humidity` from a raw measurement result.
    pub const fn from_raw(raw: u16) -> Self {
        Self(convert_humidity(raw))
    }

    /// Return relative humidity in 1/100 %RH
    pub const fn as_10mk_percent(&self) -> u16 {
        (self.0 / 10).unsigned_abs() as u16
    }

    /// Return relative humidity in 1/1000 %RH.
    pub const fn as_millipercent(&self) -> i32 {
        self.0
    }

    /// Return relative humidity in 1 %RH
    pub const fn as_1k_percent(&self) -> u8 {
        (self.0 / 1000).unsigned_abs() as u8
    }

    /// Return relative humidity in %RH.
    pub const fn as_percent(&self) -> f32 {
        self.0 as f32 / 1000.0
    }
}

/// Convert raw temperature measurement to milli-degrees celsius.
///
/// Formula (datasheet 5.11): -45 + 175 * (val / 2^16),
/// optimized for fixed point math.
#[inline]
const fn convert_temperature(temp_raw: u16) -> i32 {
    (((temp_raw as u32) * 21875) >> 13) as i32 - 45000
}

/// Convert raw humidity measurement to relative humidity.
///
/// Formula (datasheet 5.11): 100 * (val / 2^16),
/// optimized for fixed point math.
#[inline]
const fn convert_humidity(humi_raw: u16) -> i32 {
    (((humi_raw as u32) * 12500) >> 13) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test conversion of raw measurement results into °C.
    #[test]
    fn test_convert_temperature() {
        let test_data = [
            (0x0000, -45000),
            // Datasheet setion 5.11 "Conversion of Sensor Output"
            ((0b0110_0100_u16 << 8) | 0b1000_1011, 23730),
        ];
        for td in &test_data {
            assert_eq!(convert_temperature(td.0), td.1);
        }
    }

    /// Test conversion of raw measurement results into %RH.
    #[test]
    fn test_convert_humidity() {
        let test_data = [
            (0x0000, 0),
            // Datasheet setion 5.11 "Conversion of Sensor Output"
            ((0b1010_0001_u16 << 8) | 0b0011_0011, 62968),
        ];
        for td in &test_data {
            assert_eq!(convert_humidity(td.0), td.1);
        }
    }

    /// Test conversion of raw measurement results into °C and %RH.
    #[test]
    fn measurement_conversion() {
        // Datasheet setion 5.11 "Conversion of Sensor Output"
        let temperature = convert_temperature((0b0110_0100_u16 << 8) | 0b1000_1011);
        let humidity = convert_humidity((0b1010_0001_u16 << 8) | 0b0011_0011);
        assert_eq!(temperature, 23730);
        assert_eq!(humidity, 62968);
    }

    #[test]
    fn temperature() {
        let temp = Temperature(24123);
        assert_eq!(temp.as_millidegrees_celsius(), 24123);
        assert_eq!(temp.as_degrees_celsius(), 24.123);
    }

    #[test]
    fn humidity() {
        let humi = Humidity(65432);
        assert_eq!(humi.as_millipercent(), 65432);
        assert_eq!(humi.as_percent(), 65.432);
    }

    #[test]
    fn measurement_from_into() {
        // Datasheet setion 5.11 "Conversion of Sensor Output"
        let raw = RawMeasurement {
            temperature: (0b0110_0100_u16 << 8) | 0b1000_1011,
            humidity: (0b1010_0001_u16 << 8) | 0b0011_0011,
        };

        // std::convert::From
        let measurement1 = Measurement::from(raw);
        assert_eq!(measurement1.temperature.0, 23730);
        assert_eq!(measurement1.humidity.0, 62968);

        // std::convert::Into
        let measurement2: Measurement = raw.into();
        assert_eq!(measurement2.temperature.0, 23730);
        assert_eq!(measurement2.humidity.0, 62968);

        // std::cmp::PartialEq
        assert_eq!(measurement1, measurement2);
    }
}
