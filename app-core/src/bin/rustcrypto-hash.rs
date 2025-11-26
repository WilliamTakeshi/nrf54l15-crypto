#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac as _;
use panic_probe as _;
use sha2::{Digest, Sha256};

#[entry]
fn main() -> ! {
    // Start
    let msg = b"example";
    // Create a hasher and feed data
    let mut hasher = Sha256::new();
    hasher.update(msg);
    // Read result
    let digest = hasher.finalize();

    // Finish

    info!("SHA256(AAA) = {:x}", digest.as_slice());

    loop {
        cortex_m::asm::nop();
    }
}
