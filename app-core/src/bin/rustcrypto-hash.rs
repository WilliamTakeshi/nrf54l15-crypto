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
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    p.global_p2_s.pin_cnf(8).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(10).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(7).write(|w| w.dir().output());
    p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
    p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

    loop {
        p.global_p2_s.outset().write(|w| w.pin8().bit(true));
        p.global_p2_s.outset().write(|w| w.pin10().bit(true));
        let mut hasher = Sha256::new();
        hasher.update(msg);
        let digest = hasher.finalize();
        p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
        p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

        // Finish

        info!("SHA256(AAA) = {:x}", digest.as_slice());

        for _ in 0..200_000 {
            cortex_m::asm::nop();
        }
    }

    // loop {
    //     cortex_m::asm::nop();
    // }
}
