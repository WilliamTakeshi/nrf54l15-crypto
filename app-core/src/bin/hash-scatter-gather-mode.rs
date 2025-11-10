#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 CryptoMaster SHA example...");

    let input = b"example";
    info!("input: {:02x}", input);

    let mut out_sha1 = [0u8; 20];
    app_core::cracen_sha1(input, &mut out_sha1).unwrap();
    info!("output bytes SHA1: {:02x}", out_sha1);

    let mut out_sha224 = [0u8; 28];
    app_core::cracen_sha224(input, &mut out_sha224).unwrap();
    info!("output bytes SHA2_224: {:02x}", out_sha224);

    let mut out_sha256 = [0u8; 32];
    app_core::cracen_sha256(input, &mut out_sha256).unwrap();
    info!("output bytes SHA2_256: {:02x}", out_sha256);

    let mut out_sha384 = [0u8; 48];
    app_core::cracen_sha384(input, &mut out_sha384).unwrap();
    info!("output bytes SHA2_384: {:02x}", out_sha384);

    let mut out_sha512 = [0u8; 64];
    app_core::cracen_sha512(input, &mut out_sha512).unwrap();
    info!("output bytes SHA2_512: {:02x}", out_sha512);

    loop {
        cortex_m::asm::nop();
    }
}
