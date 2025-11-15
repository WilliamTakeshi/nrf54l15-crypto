#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 CryptoMaster SHA example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();
    p.global_p2_s.pin_cnf(8).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(10).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(7).write(|w| w.dir().output());

    const MESSAGE: [u8; 32] = [
        0xA4, 0x1A, 0x41, 0xA1, 0x2A, 0x79, 0x95, 0x48, 0x21, 0x1C, 0x41, 0x0C, 0x65, 0xD8, 0x13,
        0x3A, 0xFD, 0xE3, 0x4D, 0x28, 0xBD, 0xD5, 0x42, 0xE4, 0xB6, 0x80, 0xCF, 0x28, 0x99, 0xC8,
        0xA8, 0xC4,
    ];
    info!("input: {:02x}", MESSAGE.as_slice());

    // let mut out_sha1 = [0u8; 20];
    // app_core::cracen_sha1(&p, input, &mut out_sha1).unwrap();
    // info!("output bytes SHA1: {:02x}", out_sha1);

    // let mut out_sha224 = [0u8; 28];
    // app_core::cracen_sha224(&p, input, &mut out_sha224).unwrap();
    // info!("output bytes SHA2_224: {:02x}", out_sha224);

    let mut out_sha256 = [0u8; 32];

    // let mut out_sha384 = [0u8; 48];
    // app_core::cracen_sha384(&p, input, &mut out_sha384).unwrap();
    // info!("output bytes SHA2_384: {:02x}", out_sha384);

    // let mut out_sha512 = [0u8; 64];
    // app_core::cracen_sha512(&p, input, &mut out_sha512).unwrap();
    // info!("output bytes SHA2_512: {:02x}", out_sha512);

    loop {
        p.global_p2_s.outset().write(|w| w.pin8().bit(true));
        p.global_p2_s.outset().write(|w| w.pin10().bit(true));

        app_core::cracen_sha256(&p, MESSAGE.as_slice(), &mut out_sha256).unwrap();

        p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
        p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

        info!("here2{:x}", out_sha256);

        for _ in 0..100_000 {
            cortex_m::asm::nop();
        }
    }
}
