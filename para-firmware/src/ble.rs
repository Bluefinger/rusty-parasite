use bt_hci::cmd::SyncCmd;
use embassy_futures::join::join;
use embassy_nrf::{mode, pac, peripherals, rng};
use embassy_time::{Duration, Timer};
use nrf_mpsl::MultiprotocolServiceLayer;
use nrf_sdc::vendor::ZephyrWriteBdAddr;
use para_bthome::BtHomeAd;
use para_fmt::{info, unwrap};
use trouble_host::prelude::*;

use crate::{
    constants::{
        PARA_ADV_DURATION_SECS, PARA_BLE_TX_POWER, PARA_MAX_ADV_INTERVAL_MS,
        PARA_MIN_ADV_INTERVAL_MS, PARA_NAME,
    },
    state::{ADC_MEASUREMENT, SHTC3_MEASUREMENT, START_MEASUREMENTS},
};

#[embassy_executor::task]
pub async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

pub fn build_sdc<'d, const N: usize>(
    p: nrf_sdc::Peripherals<'d>,
    rng: &'d mut rng::Rng<peripherals::RNG, mode::Async>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut nrf_sdc::Mem<N>,
) -> Result<nrf_sdc::SoftdeviceController<'d>, nrf_sdc::Error> {
    nrf_sdc::Builder::new()?
        .support_adv()?
        .build(p, rng, mpsl, mem)
}

fn build_addr() -> BdAddr {
    let ficr = pac::FICR;
    let high = u64::from(ficr.deviceid(1).read());
    let addr = high << 32 | u64::from(ficr.deviceid(0).read());
    let addr = addr | 0x0000_c000_0000_0000;
    BdAddr::new(unwrap!(addr.to_le_bytes()[..6].try_into()))
}

#[embassy_executor::task]
pub async fn run(controller: nrf_sdc::SoftdeviceController<'static>) {
    let addr = build_addr();

    info!("Our address = {:?}", &addr);

    // Set the bluetooth address
    unwrap!(ZephyrWriteBdAddr::new(addr).exec(&controller).await);

    let mut resources: HostResources<DefaultPacketPool, 0, 0> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources);
    let Host {
        mut peripheral,
        mut runner,
        ..
    } = stack.build();

    let _ = join(runner.run(), async {
        let mut start_measurements = unwrap!(START_MEASUREMENTS.receiver());

        let params: AdvertisementParameters = AdvertisementParameters {
            interval_min: Duration::from_millis(PARA_MIN_ADV_INTERVAL_MS),
            interval_max: Duration::from_millis(PARA_MAX_ADV_INTERVAL_MS),
            tx_power: PARA_BLE_TX_POWER,
            ..Default::default()
        };

        loop {
            start_measurements.changed().await;

            let (adc, shtc3) = join(ADC_MEASUREMENT.wait(), SHTC3_MEASUREMENT.wait()).await;

            let mut ad = BtHomeAd::default();

            let adv_data = ad
                .add_data(adc.battery)
                .add_data(shtc3.temperature)
                .add_data(adc.lux)
                .add_data(adc.voltage)
                .add_data(shtc3.humidity)
                .add_data(adc.moisture)
                .add_local_name(PARA_NAME)
                .encode();

            info!("Starting advertising");
            let advertiser = unwrap!(
                peripheral
                    .advertise(
                        &params,
                        Advertisement::NonconnectableScannableUndirected {
                            adv_data,
                            scan_data: &[],
                        },
                    )
                    .await
            );
            Timer::after_secs(PARA_ADV_DURATION_SECS).await;
            drop(advertiser);
            info!("Stopping advertising, sleeping...");
        }
    })
    .await;
}
