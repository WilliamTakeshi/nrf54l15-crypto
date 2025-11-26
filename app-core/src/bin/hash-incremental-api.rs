#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 CryptoMaster SHA example...");
    // let p = nrf54l15_app_pac::Peripherals::take().unwrap();
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    let mut state = app_core::HashState::<32, 128>::init(app_core::HashAlg::Sha2_256);
    state.update(b"exam");
    state.update(b"ple");
    let out = state.finalize(&p);

    info!("out {:02x}", out.unwrap());

    loop {
        cortex_m::asm::nop();
    }
}
