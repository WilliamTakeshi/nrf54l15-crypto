#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
// use nrf54l15_app_pac;
use nrf54l15_flpr_pac;
use panic_probe as _;
use riscv::{self as _, delay::McycleDelay};
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let p = nrf54l15_flpr_pac::Peripherals::take().unwrap();
    // let p = nrf54l15_flpr_pac::Peripherals::take().unwrap();

    // 1. Enable the CRACEN global block
    let cracen = &p.global_cracen_s;
    // Assuming the PAC provides enable/disable fields
    // (some variants use `enable` register, others `enable_set`)
    unsafe {
        cracen.enable().write(|w| w.bits(1)); // or .set_bit() if defined
    }

    // 2. Access RNG control block
    let rng = p.global_cracencore_s.rngcontrol();

    // 3. Enable RNG and start generation
    // Check which bits exist in CONTROL (enable/start)
    unsafe {
        rng.control().write(|w| w.bits(1)); // Bit 0 = enable/start (common pattern)
    }

    // // Wait for RNG to have data available
    // while rng.fifolevel().read().bits() == 0 {
    //     cortex_m::asm::nop();
    // }

    // // 4. Read a few random words from FIFO
    // let mut rand_words = [0u32; 4];
    // for (i, word) in rand_words.iter_mut().enumerate() {
    //     *word = rng.fifo(i).read().bits();
    //     info!("word: {}", word);
    // }

    // At this point, rand_words contains true random values
    loop {
        cortex_m::asm::nop();
    }
}
