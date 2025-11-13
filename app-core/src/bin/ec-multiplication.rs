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

    // Enable RNG
    p.global_cracen_s.enable().write(|w| {
        w.rng().set_bit();
        w.cryptomaster().set_bit();
        w.pkeikg().set_bit()
    });

    unsafe { app_core::load_microcode() };

    let cracen = p.global_cracencore_s;

    while cracen.pk().status().read().pkbusy().bit_is_set() {}
    while cracen.ikg().status().read().ctrdrbgbusy().bit_is_set() {}

    // TODO: shrink this unsafe
    unsafe {
        // cracen.pk().command().write(|w| w.bits(0x10101F22)); // 269492002
        cracen.pk().command().write(|w| {
            w.opeaddr().bits(0x22);
            w.opbytesm1().bits(0b0000011111);
            w.selcurve().p256();
            w.swapbytes().set_bit()
        });

        while cracen.pk().status().read().pkbusy().bit_is_set() {}
        while cracen.ikg().status().read().ctrdrbgbusy().bit_is_set() {}

        let scalar: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x04,
        ];
        app_core::write_block::<32>(app_core::slot_addr(8), &scalar); // Block 8

        // k=3
        // 5ECBE4D1A6330A44C8F7EF951D4BF165E6C6B721EFADA985FB41661BC6E7FD6C
        let pub_key_x: [u8; 32] = [
            0x5E, 0xCB, 0xE4, 0xD1, 0xA6, 0x33, 0x0A, 0x44, 0xC8, 0xF7, 0xEF, 0x95, 0x1D, 0x4B,
            0xF1, 0x65, 0xE6, 0xC6, 0xB7, 0x21, 0xEF, 0xAD, 0xA9, 0x85, 0xFB, 0x41, 0x66, 0x1B,
            0xC6, 0xE7, 0xFD, 0x6C,
        ];
        app_core::write_block::<32>(app_core::slot_addr(12), &pub_key_x); // Block 12

        // k=3
        // 8734640C4998FF7E374B06CE1A64A2ECD82AB036384FB83D9A79B127A27D5032
        let pub_key_y: [u8; 32] = [
            0x87, 0x34, 0x64, 0x0C, 0x49, 0x98, 0xFF, 0x7E, 0x37, 0x4B, 0x06, 0xCE, 0x1A, 0x64,
            0xA2, 0xEC, 0xD8, 0x2A, 0xB0, 0x36, 0x38, 0x4F, 0xB8, 0x3D, 0x9A, 0x79, 0xB1, 0x27,
            0xA2, 0x7D, 0x50, 0x32,
        ];
        app_core::write_block::<32>(app_core::slot_addr(13), &pub_key_y); // Block 13

        cracen.pk().pointers().write(|w| {
            w.opptra().bits(12);
            w.opptrb().bits(8);
            w.opptrc().bits(10)
        });

        cracen.pk().control().write(|w| {
            w.start().set_bit();
            w.clearirq().set_bit()
        });
    }

    while cracen.pk().status().read().pkbusy().bit_is_set() {}
    while cracen.ikg().status().read().ctrdrbgbusy().bit_is_set() {}
    info!("Done");

    let bytes = unsafe { app_core::read32_bytes(app_core::slot_addr(10)) };
    info!("P-256 X: {:02x}", bytes);

    let bytes = unsafe { app_core::read32_bytes(app_core::slot_addr(11)) };
    info!("P-256 Y: {:02x}", bytes);

    loop {
        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }
}
