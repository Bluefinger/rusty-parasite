use embassy_nrf::gpio::Input;
use embassy_time::Timer;

use crate::state::START_MEASUREMENTS;

#[embassy_executor::task]
pub async fn task(mut btn: Input<'static>) {
    let measure = START_MEASUREMENTS.sender();

    loop {
        btn.wait_for_rising_edge().await;
        measure.send(());
        Timer::after_secs(5).await;
    }
}
