#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 blinky example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();
    p.global_p2_s.pin_cnf(9).write(|w| w.dir().output());
    p.global_p1_s.pin_cnf(10).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(7).write(|w| w.dir().output());
    p.global_p1_s.pin_cnf(14).write(|w| w.dir().output());

    loop {
        // Turn LED on
        p.global_p2_s.outset().write(|w| w.pin9().bit(true));
        p.global_p1_s.outset().write(|w| w.pin10().bit(true));
        p.global_p2_s.outset().write(|w| w.pin7().bit(true));
        p.global_p1_s.outset().write(|w| w.pin14().bit(true));

        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }

        // Turn LED off
        p.global_p2_s.outclr().write(|w| w.pin9().bit(true));
        p.global_p1_s.outclr().write(|w| w.pin10().bit(true));
        p.global_p2_s.outclr().write(|w| w.pin7().bit(true));
        p.global_p1_s.outclr().write(|w| w.pin14().bit(true));

        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }
}
