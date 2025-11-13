#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 RNG buffer example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    let mut buf = [0u8; 64];
    app_core::rng(&p, &mut buf);

    info!("RNG buffer:");
    info!("buf: {:02x}", buf);

    loop {
        cortex_m::asm::nop();
    }
}
