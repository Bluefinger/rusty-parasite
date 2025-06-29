use embassy_time::{Duration, Ticker, Timer};

use crate::{constants::PARA_SLEEP_SECS, state::START_MEASUREMENTS};

#[embassy_executor::task]
pub async fn task() {
    let mut ticker = Ticker::every(Duration::from_secs(PARA_SLEEP_SECS));
    let start_measurements = START_MEASUREMENTS.sender();

    Timer::after_secs(1).await;

    loop {
        start_measurements.send(());
        ticker.next().await;
    }
}