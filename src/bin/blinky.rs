#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // https://docs.nordicsemi.com/bundle/ug_nrf54l15_dk/page/UG/nRF54L15_DK/hw_desription/buttons_leds.html
    let mut led1 = Output::new(p.P2_09, Level::Low, OutputDrive::Standard);
    let mut led2 = Output::new(p.P1_10, Level::Low, OutputDrive::Standard);
    let mut led3 = Output::new(p.P2_07, Level::Low, OutputDrive::Standard);
    let mut led4 = Output::new(p.P1_14, Level::Low, OutputDrive::Standard);

    loop {
        led1.set_high();
        led2.set_high();
        led3.set_high();
        led4.set_high();
        Timer::after_millis(100).await;
        led1.set_low();
        led2.set_low();
        led3.set_low();
        led4.set_low();
        Timer::after_millis(100).await;
    }
}
