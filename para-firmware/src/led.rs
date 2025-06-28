use embassy_nrf::gpio::Output;
use embassy_time::Timer;
use para_fmt::unwrap;

use crate::state::START_MEASUREMENTS;

#[embassy_executor::task]
pub async fn task(mut led: Output<'static>) {
    let mut indication = unwrap!(START_MEASUREMENTS.receiver());

    loop {
        indication.changed().await;
        for _ in 0..4 {
            led.set_high();
            Timer::after_millis(50).await;
            led.set_low();
            Timer::after_millis(450).await;
        }
    }
}
