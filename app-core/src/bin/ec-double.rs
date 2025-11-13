#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 EC-ADD example...");
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
        cracen.pk().command().write(|w| {
            w.opeaddr().bits(0x20);
            w.opbytesm1().bits(0b0000011111);
            w.selcurve().p256();
            w.swapbytes().set_bit()
        });

        while cracen.pk().status().read().pkbusy().bit_is_set() {}
        while cracen.ikg().status().read().ctrdrbgbusy().bit_is_set() {}

        // k = 3
        // x = 5ECBE4D1A6330A44C8F7EF951D4BF165E6C6B721EFADA985FB41661BC6E7FD6C
        let _pub_x_k3: [u8; 32] = [
            0x5E, 0xCB, 0xE4, 0xD1, 0xA6, 0x33, 0x0A, 0x44, 0xC8, 0xF7, 0xEF, 0x95, 0x1D, 0x4B,
            0xF1, 0x65, 0xE6, 0xC6, 0xB7, 0x21, 0xEF, 0xAD, 0xA9, 0x85, 0xFB, 0x41, 0x66, 0x1B,
            0xC6, 0xE7, 0xFD, 0x6C,
        ];

        // k = 3
        // y = 8734640C4998FF7E374B06CE1A64A2ECD82AB036384FB83D9A79B127A27D5032
        let _pub_y_k3: [u8; 32] = [
            0x87, 0x34, 0x64, 0x0C, 0x49, 0x98, 0xFF, 0x7E, 0x37, 0x4B, 0x06, 0xCE, 0x1A, 0x64,
            0xA2, 0xEC, 0xD8, 0x2A, 0xB0, 0x36, 0x38, 0x4F, 0xB8, 0x3D, 0x9A, 0x79, 0xB1, 0x27,
            0xA2, 0x7D, 0x50, 0x32,
        ];

        // k = 4
        // x = E2534A3532D08FBBA02DDE659EE62BD0031FE2DB785596EF509302446B030852
        let _pub_x_k4: [u8; 32] = [
            0xE2, 0x53, 0x4A, 0x35, 0x32, 0xD0, 0x8F, 0xBB, 0xA0, 0x2D, 0xDE, 0x65, 0x9E, 0xE6,
            0x2B, 0xD0, 0x03, 0x1F, 0xE2, 0xDB, 0x78, 0x55, 0x96, 0xEF, 0x50, 0x93, 0x02, 0x44,
            0x6B, 0x03, 0x08, 0x52,
        ];
        // k = 4
        // y = E0F1575A4C633CC719DFEE5FDA862D764EFC96C3F30EE0055C42C23F184ED8C6
        let _pub_y_k4: [u8; 32] = [
            0xE0, 0xF1, 0x57, 0x5A, 0x4C, 0x63, 0x3C, 0xC7, 0x19, 0xDF, 0xEE, 0x5F, 0xDA, 0x86,
            0x2D, 0x76, 0x4E, 0xFC, 0x96, 0xC3, 0xF3, 0x0E, 0xE0, 0x05, 0x5C, 0x42, 0xC2, 0x3F,
            0x18, 0x4E, 0xD8, 0xC6,
        ];

        // k = 5
        // x = 51590B7A515140D2D784C85608668FDFEF8C82FD1F5BE52421554A0DC3D033ED
        let pub_x_k5: [u8; 32] = [
            0x51, 0x59, 0x0B, 0x7A, 0x51, 0x51, 0x40, 0xD2, 0xD7, 0x84, 0xC8, 0x56, 0x08, 0x66,
            0x8F, 0xDF, 0xEF, 0x8C, 0x82, 0xFD, 0x1F, 0x5B, 0xE5, 0x24, 0x21, 0x55, 0x4A, 0x0D,
            0xC3, 0xD0, 0x33, 0xED,
        ];
        // k = 5
        // y = E0C17DA8904A727D8AE1BF36BF8A79260D012F00D4D80888D1D0BB44FDA16DA4
        let pub_y_k5: [u8; 32] = [
            0xE0, 0xC1, 0x7D, 0xA8, 0x90, 0x4A, 0x72, 0x7D, 0x8A, 0xE1, 0xBF, 0x36, 0xBF, 0x8A,
            0x79, 0x26, 0x0D, 0x01, 0x2F, 0x00, 0xD4, 0xD8, 0x08, 0x88, 0xD1, 0xD0, 0xBB, 0x44,
            0xFD, 0xA1, 0x6D, 0xA4,
        ];

        app_core::write_block::<32>(app_core::slot_addr(12), &pub_x_k5);
        app_core::write_block::<32>(app_core::slot_addr(13), &pub_y_k5);

        cracen.pk().pointers().write(|w| {
            w.opptra().bits(12);
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
        info!(
            "errorflags: {:b}, failptr: {:b}, pkbusy: {}, intrptstatus: {}",
            cracen.pk().status().read().errorflags().bits(),
            cracen.pk().status().read().failptr().bits(),
            cracen.pk().status().read().pkbusy().bit_is_set(),
            cracen.pk().status().read().intrptstatus().bit_is_set()
        );

        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }
}
