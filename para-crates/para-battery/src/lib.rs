//! Crate for calculating Battery levels as percentages, based on voltage/pct profiles via
//! [`BatteryDischargeProfile`].
#![no_std]

use core::ops::Range;

pub struct BatteryDischargeProfile {
    voltage_range: Range<f32>,
    pct_range: Range<f32>,
}

impl BatteryDischargeProfile {
    /// Creates a new discharge profile. Internally, it stores the voltages high/low and pct high/low
    /// as ranges.
    #[inline]
    pub const fn new(voltage_high: f32, voltage_low: f32, pct_high: f32, pct_low: f32) -> Self {
        Self {
            voltage_range: voltage_low..voltage_high,
            pct_range: pct_low..pct_high,
        }
    }

    /// Calculates a battery percentage according to the specified range of the discharge profile.
    /// If the voltage is outside of the discharge profile, this method returns `None`.
    /// 
    /// ```
    /// use para_battery::BatteryDischargeProfile;
    ///
    /// let level = BatteryDischargeProfile::new(3.0, 2.0, 1.0, 0.0);
    ///
    /// assert_eq!(level.calc_pct(2.5), Some(0.5));
    /// ```
    pub fn calc_pct(&self, voltage: f32) -> Option<f32> {
        if self.voltage_range.contains(&voltage) {
            Some(
                self.pct_range.start
                    + (voltage - self.voltage_range.start)
                        * ((self.pct_range.end - self.pct_range.start)
                            / (self.voltage_range.end - self.voltage_range.start)),
            )
        } else {
            None
        }
    }

    /// Calculates a battery level from a range of discharge profiles. Assumes the first
    /// discharge level is the highest, so the levels go from high to low. Percentages values
    /// are from 1.0 to 0.0.
    ///
    /// ```
    /// use para_battery::BatteryDischargeProfile;
    /// 
    /// let levels = [
    ///     BatteryDischargeProfile::new(3.0, 2.5, 1.0, 0.5),
    ///     BatteryDischargeProfile::new(2.5, 2.0, 0.5, 0.0),
    /// ];
    ///
    /// assert_eq!(BatteryDischargeProfile::calc_pct_from_profile_range(2.75, levels.iter()), 0.75);
    /// ```
    pub fn calc_pct_from_profile_range<'a>(
        voltage: f32,
        levels: impl Iterator<Item = &'a BatteryDischargeProfile>,
    ) -> f32 {
        let mut levels = levels.peekable();

        if levels
            .peek()
            .is_some_and(|&level| voltage >= level.voltage_range.end)
        {
            return 1.0;
        }

        levels
            .find_map(|level| level.calc_pct(voltage))
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_level_from_one_profile() {
        let level = BatteryDischargeProfile::new(3.0, 2.0, 1.0, 0.0);

        assert_eq!(level.calc_pct(2.5), Some(0.5));
        assert_eq!(level.calc_pct(3.5), None);
        assert_eq!(level.calc_pct(1.5), None);
    }

    #[test]
    fn battery_level_from_profile_range() {
        let levels = [
            BatteryDischargeProfile::new(3.0, 2.5, 1.0, 0.5),
            BatteryDischargeProfile::new(2.5, 2.0, 0.5, 0.0),
        ];

        let expect_results: [(f32, f32); 4] = [(3.5, 1.0), (2.75, 0.75), (2.25, 0.25), (1.5, 0.0)];

        for (voltage, pct) in expect_results {
            assert_eq!(
                BatteryDischargeProfile::calc_pct_from_profile_range(voltage, levels.iter()),
                pct
            );
        }
    }
}
