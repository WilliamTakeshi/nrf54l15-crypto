#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 EC-multiplication example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    p.global_cracen_s.enable().write(|w| {
        w.rng().set_bit();
        w.cryptomaster().set_bit();
        w.pkeikg().set_bit()
    });

    unsafe { app_core::load_microcode() };

    let cracen = &p.global_cracencore_s;

    while cracen.pk().status().read().pkbusy().bit_is_set() {}
    while cracen.ikg().status().read().ctrdrbgbusy().bit_is_set() {}

    let mut out_x = [0u8; 32];
    let mut out_y = [0u8; 32];
    let scalar: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x04,
    ];
    // k=3
    // 5ECBE4D1A6330A44C8F7EF951D4BF165E6C6B721EFADA985FB41661BC6E7FD6C
    let pub_key_x: [u8; 32] = [
        0x5E, 0xCB, 0xE4, 0xD1, 0xA6, 0x33, 0x0A, 0x44, 0xC8, 0xF7, 0xEF, 0x95, 0x1D, 0x4B, 0xF1,
        0x65, 0xE6, 0xC6, 0xB7, 0x21, 0xEF, 0xAD, 0xA9, 0x85, 0xFB, 0x41, 0x66, 0x1B, 0xC6, 0xE7,
        0xFD, 0x6C,
    ];
    // k=3
    // 8734640C4998FF7E374B06CE1A64A2ECD82AB036384FB83D9A79B127A27D5032
    let pub_key_y: [u8; 32] = [
        0x87, 0x34, 0x64, 0x0C, 0x49, 0x98, 0xFF, 0x7E, 0x37, 0x4B, 0x06, 0xCE, 0x1A, 0x64, 0xA2,
        0xEC, 0xD8, 0x2A, 0xB0, 0x36, 0x38, 0x4F, 0xB8, 0x3D, 0x9A, 0x79, 0xB1, 0x27, 0xA2, 0x7D,
        0x50, 0x32,
    ];

    p.global_p2_s.pin_cnf(8).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(10).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(7).write(|w| w.dir().output());
    p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
    p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

    loop {
        let mut out_x = [0u8; 32];
        let mut out_y = [0u8; 32];

        p.global_p2_s.outset().write(|w| w.pin8().bit(true));
        p.global_p2_s.outset().write(|w| w.pin10().bit(true));

        app_core::cracen_ec_scalar_mul(&p, &scalar, &pub_key_x, &pub_key_y, &mut out_x, &mut out_y);

        p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
        p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

        info!("Result X = {:02x}", out_x);
        info!("Result Y = {:02x}", out_y);

        for _ in 0..100_000 {
            cortex_m::asm::nop();
        }
    }

    // loop {
    //     cortex_m::asm::nop();
    // }
}
