#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 RNG example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    // 1. Enable RNG
    p.global_cracen_s.enable().write(|w| w.rng().set_bit());

    // 2. Start RNG
    p.global_cracencore_s
        .rngcontrol()
        .control()
        .write(|w| w.enable().set_bit());

    loop {
        let random_word = p.global_cracencore_s.rngcontrol().fifo(0).read().bits();
        info!("random_word: {}", random_word);

        for _ in 0..100_000 {
            cortex_m::asm::nop();
        }
    }
}
