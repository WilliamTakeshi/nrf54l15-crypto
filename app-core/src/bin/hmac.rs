#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 CryptoMaster HMAC example...");

    let key = b"supersecretkey";
    let message = b"hello world";

    let mut tag = [0u8; 32];
    app_core::cracen_hmac_sha256(key, message, &mut tag).unwrap();

    info!("HMAC-SHA256: {:02x}", tag);

    loop {
        cortex_m::asm::nop();
    }
}
