#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
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
        // 3. Wait for data to be available in the FIFO
        while p.global_cracencore_s.rngcontrol().fifolevel().read().bits() == 0 {}

        // 4. Read random word from FIFO
        for i in 0..16 {
            let random_word = p.global_cracencore_s.rngcontrol().fifo(i).read().bits();
            info!("random_word[{:02}]: {:08x}", i, random_word);
        }
        info!("-------------------------");

        // 5. Short delay between reads
        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }
}
