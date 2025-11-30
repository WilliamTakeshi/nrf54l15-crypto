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
    let fst_upd: [u8; 2] = [98; 2];
    let snd_upd: [u8; 130] = [98; 130];

    let mut state = app_core::HashState::init(app_core::HashAlg::Sha2_256);
    info!("initial_state: {}", state);
    state.update(&fst_upd);
    // info!("state after fst update: {}", state);
    // state.update(&snd_upd);
    // info!("state after snd update: {}", state);
    let out = state.finalize(&p);

    info!("out: {:02x}", out);

    let mut state = app_core::HashState::init(app_core::HashAlg::Sha2_256);
    info!("initial_state: {}", state);
    state.update(&fst_upd);
    // info!("state after fst update: {}", state);
    state.update(&snd_upd);
    // info!("state after snd update: {}", state);
    let out = state.finalize(&p);

    info!("out: {:02x}", out);

    info!("DONE");

    // info!("out {:02x}", out.unwrap());

    loop {
        cortex_m::asm::nop();
    }
}
