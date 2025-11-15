#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 ECDSA example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    // Enable RNG
    p.global_cracen_s.enable().write(|w| {
        w.rng().set_bit();
        w.cryptomaster().set_bit();
        w.pkeikg().set_bit()
    });
    unsafe { app_core::load_microcode() };

    while p
        .global_cracencore_s
        .pk()
        .status()
        .read()
        .pkbusy()
        .bit_is_set()
    {}
    while p
        .global_cracencore_s
        .ikg()
        .status()
        .read()
        .ctrdrbgbusy()
        .bit_is_set()
    {}

    let priv_key: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x02,
    ];

    let pub_key_x: [u8; 32] = [
        0x7c, 0xf2, 0x7b, 0x18, 0x8d, 0x03, 0x4f, 0x7e, 0x8a, 0x52, 0x38, 0x03, 0x04, 0xb5, 0x1a,
        0xc3, 0xc0, 0x89, 0x69, 0xe2, 0x77, 0xf2, 0x1b, 0x35, 0xa6, 0x0b, 0x48, 0xfc, 0x47, 0x66,
        0x99, 0x78,
    ];

    let pub_key_y: [u8; 32] = [
        0x07, 0x77, 0x55, 0x10, 0xdb, 0x8e, 0xd0, 0x40, 0x29, 0x3d, 0x9a, 0xc6, 0x9f, 0x74, 0x30,
        0xdb, 0xba, 0x7d, 0xad, 0xe6, 0x3c, 0xe9, 0x82, 0x29, 0x9e, 0x04, 0xb7, 0x9d, 0x22, 0x78,
        0x73, 0xd1,
    ];

    let msg = b"example";

    p.global_p2_s.pin_cnf(8).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(10).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(7).write(|w| w.dir().output());
    p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
    p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

    loop {
        p.global_p2_s.outset().write(|w| w.pin8().bit(true));
        p.global_p2_s.outset().write(|w| w.pin10().bit(true));
        let (bytes_x, bytes_y) = app_core::cracen_ecdsa_sign(&p, msg, &priv_key).unwrap();
        p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
        p.global_p2_s.outclr().write(|w| w.pin10().bit(true));
        info!("Done");

        // Printing Result
        let mut buf = [0u8; 128]; // 64 bytes * 2 hex chars
        let mut pos = 0;
        for b in bytes_x {
            pos = push_hex(&mut buf, pos, b);
        }
        for b in bytes_y {
            pos = push_hex(&mut buf, pos, b);
        }

        info!("ECDSA sign: {}", core::str::from_utf8(&buf[..]).unwrap());
        p.global_p2_s.outset().write(|w| w.pin8().bit(true));
        p.global_p2_s.outset().write(|w| w.pin10().bit(true));

        let verified =
            app_core::cracen_ecdsa_verify(&p, msg, &bytes_x, &bytes_y, &pub_key_x, &pub_key_y);

        p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
        p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

        if verified {
            info!("Signature verified successfully");
        } else {
            info!("Signature verification failed");
        }

        for _ in 0..200_000 {
            cortex_m::asm::nop();
        }
    }

    // loop {
    //     cortex_m::asm::nop();
    // }
}

fn push_hex(buf: &mut [u8], pos: usize, val: u8) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    if pos + 2 <= buf.len() {
        buf[pos] = HEX[(val >> 4) as usize];
        buf[pos + 1] = HEX[(val & 0x0F) as usize];
    }
    pos + 2
}
