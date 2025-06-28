use embassy_nrf::gpio::Output;
use embassy_time::Timer;

use crate::state::START_MEASUREMENTS;

#[embassy_executor::task]
pub async fn task(mut led: Output<'static>) {
    let mut indication = START_MEASUREMENTS.receiver().unwrap();
    loop {
        indication.changed().await;
        for _ in 0..3 {
            led.set_high();
            Timer::after_millis(50).await;
            led.set_low();
            Timer::after_millis(450).await;
        }
    }
}
