#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 RNG example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    //  Enable
    p.global_cracen_s.enable().write(|w| {
        w.rng().set_bit();
        w.pkeikg().set_bit();
        w.cryptomaster().set_bit()
    });
    // Wait.

    let cracen = p.global_cracencore_s;

    while cracen.pk().status().read().pkbusy().bit_is_set() {}
    while cracen.ikg().status().read().ctrdrbgbusy().bit_is_set() {}

    unsafe {
        // command 10101F22
        cracen.pk().command().write(|w| w.bits(0x10101F22));

        // write 0x000002 on 0x518091e0 ""
        let priv_key: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];
        write_block(0x5180_91e0, &priv_key);

        // write 6b17... on 0x518099e0 ""
        let publ_key_x: [u8; 32] = [
            0x6B, 0x17, 0xD1, 0xF2, 0xE1, 0x2C, 0x42, 0x47, 0xF8, 0xBC, 0xE6, 0xE5, 0x63, 0xA4,
            0x40, 0xF2, 0x77, 0x03, 0x7D, 0x81, 0x2D, 0xEB, 0x33, 0xA0, 0xF4, 0xA1, 0x39, 0x45,
            0xD8, 0x98, 0xC2, 0x96,
        ];
        write_block(0x5180_99e0, &publ_key_x);

        // write 4f3e... on 0x51809be0 ""
        let publ_key_y: [u8; 32] = [
            0x4F, 0xE3, 0x42, 0xE2, 0xFE, 0x1A, 0x7F, 0x9B, 0x8E, 0xE7, 0xEB, 0x4A, 0x7C, 0x0F,
            0x9E, 0x16, 0x2B, 0xCE, 0x33, 0x57, 0x6B, 0x31, 0x5E, 0xCE, 0xCB, 0xB6, 0x40, 0x68,
            0x37, 0xBF, 0x51, 0xF5,
        ];
        write_block(0x5180_9be0, &publ_key_y);

        // Write Pointers (12, 8, 10)
        cracen.pk().pointers().write(|w| {
            w.opptra().bits(12);
            w.opptrb().bits(8);
            w.opptrc().bits(10)
        });

        // Wait
        while cracen.pk().status().read().pkbusy().bit_is_set() {}
        while cracen.ikg().status().read().ctrdrbgbusy().bit_is_set() {}

        cracen.pk().control().write(|w| {
            w.start().set_bit();
            w.clearirq().set_bit()
        });

        let foo = read32_bytes(0x5180_95e0);

        info!("foo: {}", foo);

        let bar = read32_bytes(0x5180_97e0);

        info!("bar: {}", bar);
    }

    info!(
        "all_error {:b},
errorflags: {:b}, pkbusy: {:b}, intrptstatus: {:b}, failptr: {:b}",
        cracen.pk().status().read().bits(),
        cracen.pk().status().read().errorflags().bits(),
        cracen.pk().status().read().pkbusy().bit_is_set(),
        cracen.pk().status().read().intrptstatus().bit_is_set(),
        cracen.pk().status().read().failptr().bits(),
    );

    // 0x518095e0 output
    // 0x518097e0

    loop {
        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }
}

unsafe fn write_block(addr: u32, data: &[u8; 32]) {
    let mut p = addr as *mut u32;
    for chunk in data.chunks_exact(4) {
        let v = u32::from_be_bytes(chunk.try_into().unwrap());
        core::ptr::write_volatile(p, v);
        p = p.add(1);
    }
}

pub unsafe fn read32_bytes(addr: u32) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut p = addr as *const u32;

    for i in 0..8 {
        let v = core::ptr::read_volatile(p);
        let bytes = v.to_be_bytes();
        out[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        p = p.add(1);
    }

    out
}
