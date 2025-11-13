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

    p.global_cracen_s.enable().write(|w| {
        w.rng().set_bit();
        w.cryptomaster().set_bit();
        w.pkeikg().set_bit()
    });

    let mut buf = [0u8; 64];
    app_core::rng(&p, &mut buf);

    info!("RNG buffer:");
    info!("buf: {:02x}", buf);

    loop {
        cortex_m::asm::nop();
    }
}
