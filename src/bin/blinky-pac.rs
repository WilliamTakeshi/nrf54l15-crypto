#![no_std]
#![no_main]

#[unsafe(link_section = ".vector_table.interrupts")]
#[unsafe(no_mangle)]
pub static __INTERRUPTS: [unsafe extern "C" fn(); 240] = [default_handler; 240];

unsafe extern "C" fn default_handler() {}

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac as pac;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 blinky example...");
    let p = pac::Peripherals::take().unwrap();
    p.global_p2_s.pin_cnf(9).write(|w| w.dir().output());

    loop {
        // Turn LED on
        p.global_p2_s.outset().write(|w| w.pin9().bit(true));

        for _ in 0..100_000 {
            cortex_m::asm::nop();
        }

        // Turn LED off
        p.global_p2_s.outclr().write(|w| w.pin9().bit(true));

        for _ in 0..100_000 {
            cortex_m::asm::nop();
        }
    }
}
