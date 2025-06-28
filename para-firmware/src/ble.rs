use bt_hci::cmd::le::*;
use bt_hci::controller::ControllerCmdSync;
use embassy_futures::join::join;
use embassy_nrf::{mode, peripherals, rng};
use embassy_time::{Duration, Timer};
use nrf_mpsl::MultiprotocolServiceLayer;
use trouble_host::prelude::*;

use crate::fmt::{info, unwrap};

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

pub async fn run<C>(controller: C)
where
    C: Controller
        + for<'t> ControllerCmdSync<LeSetExtAdvData<'t>>
        + ControllerCmdSync<LeClearAdvSets>
        + ControllerCmdSync<LeSetExtAdvParams>
        + ControllerCmdSync<LeSetAdvSetRandomAddr>
        + ControllerCmdSync<LeReadNumberOfSupportedAdvSets>
        + for<'t> ControllerCmdSync<LeSetExtAdvEnable<'t>>
        + for<'t> ControllerCmdSync<LeSetExtScanResponseData<'t>>,
{
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address);

    let mut resources: HostResources<DefaultPacketPool, 0, 0> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral,
        mut runner,
        ..
    } = stack.build();

    let mut adv_data = [0; 31];
    let len = unwrap!(AdStructure::encode_slice(
        &[
            AdStructure::CompleteLocalName(b"r-para"),
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
        ],
        &mut adv_data[..],
    ));

    let _ = join(runner.run(), async {
        let params: AdvertisementParameters = AdvertisementParameters {
            interval_min: Duration::from_millis(100),
            interval_max: Duration::from_millis(100),
            ..Default::default()
        };

        loop {
            info!("Starting advertising");
            let advertiser = unwrap!(
                peripheral
                    .advertise(
                        &params,
                        Advertisement::NonconnectableScannableUndirected {
                            adv_data: &adv_data[..len],
                            scan_data: &[],
                        },
                    )
                    .await,
                "Failed to advertise"
            );
            Timer::after_secs(4).await;
            drop(advertiser);
            info!("Stopping advertising, sleeping...");
            Timer::after(Duration::from_secs(60)).await;
        }
    })
    .await;
}
