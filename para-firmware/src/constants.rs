use para_battery::BatteryDischargeProfile;
use trouble_host::prelude::TxPower;

pub const PARA_SLEEP_SECS: u64 = 300;
pub const PARA_ADV_DURATION_SECS: u64 = 4;
pub const PARA_MIN_ADV_INTERVAL_MS: u64 = 30;
pub const PARA_MAX_ADV_INTERVAL_MS: u64 = 80;
pub const PARA_BLE_TX_POWER: TxPower = TxPower::Plus8dBm;

pub static PARA_NAME: &str = "r-para";

pub static DRY_COEFFS: [f32; 3] = [154.0, 110.0, -15.3];
pub static WET_COEFFS: [f32; 3] = [319.0, -63.1, 7.2];

pub static DISCARGE_PROFILES: [BatteryDischargeProfile; 4] = [
    BatteryDischargeProfile::new(3.00, 2.90, 1.00, 0.42),
    BatteryDischargeProfile::new(2.90, 2.74, 0.42, 0.18),
    BatteryDischargeProfile::new(2.74, 2.44, 0.18, 0.06),
    BatteryDischargeProfile::new(2.44, 2.01, 0.06, 0.00),
];
