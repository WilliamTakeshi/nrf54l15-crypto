#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac::{self, global_cracen_s};
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 ECDH example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    // Enable RNG
    p.global_cracen_s.enable().write(|w| {
        w.rng().set_bit();
        w.cryptomaster().set_bit();
        w.pkeikg().set_bit()
    });

    let cracen = p.global_cracencore_s;

    while cracen.pk().status().read().pkbusy().bit_is_set() {}
    while cracen.ikg().status().read().ctrdrbgbusy().bit_is_set() {}

    for _ in 0..1_000_000 {
        cortex_m::asm::nop();
    }

    // TODO: shrink this unsafe
    unsafe {
        // cracen.pk().command().write(|w| w.bits(0x10101F22)); // 269492002
        cracen.pk().command().write(|w| {
            w.opeaddr().bits(0x22);
            w.opbytesm1().bits(0b0000011111);
            w.selcurve().p256();
            w.swapbytes().set_bit()
        });

        info!("command: {}", cracen.pk().command().read().bits());

        while cracen.pk().status().read().pkbusy().bit_is_set() {}

        let scalar: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];

        write_block(slot_addr(8), &scalar); // Block 8

        let pub_key_x: [u8; 32] = [
            0x5E, 0xCB, 0xE4, 0xD1, 0xA6, 0x33, 0x0A, 0x44, 0xC8, 0xF7, 0xEF, 0x95, 0x1D, 0x4B,
            0xF1, 0x65, 0xE6, 0xC6, 0xB7, 0x21, 0xEF, 0xAD, 0xA9, 0x85, 0xFB, 0x41, 0x66, 0x1B,
            0xC6, 0xE7, 0xFD, 0x6C,
        ];

        write_block(slot_addr(12), &pub_key_x); // Block 12

        let pub_key_y: [u8; 32] = [
            0x87, 0x34, 0x64, 0x0C, 0x49, 0x98, 0xFF, 0x7E, 0x37, 0x4B, 0x06, 0xCE, 0x1A, 0x64,
            0xA2, 0xEC, 0xD8, 0x2A, 0xB0, 0x36, 0x38, 0x4F, 0xB8, 0x3D, 0x9A, 0x79, 0xB1, 0x27,
            0xA2, 0x7D, 0x50, 0x32,
        ];
        write_block(slot_addr(13), &pub_key_y); // Block 13

        cracen.pk().pointers().write(|w| {
            w.opptra().bits(12);
            w.opptrb().bits(8);
            w.opptrc().bits(10)
        });
        // A080C
        for _ in 0..300_000 {
            cortex_m::asm::nop();
        }

        let foo2 = cracen.pk().command().read().bits();

        info!("foo2: {:x}", foo2);

        // p.global_cracen_s
        //     .protectedramlock()
        //     .write(|w| w.enable().set_bit());

        cracen.pk().control().write(|w| {
            w.start().set_bit();
            w.clearirq().set_bit()
        });
    }

    info!("Done");

    loop {
        info!(
            "all_error {:b},
errorflags: {:b}, pkbusy: {:b}, intrptstatus: {:b}, failptr: {:b}",
            cracen.pk().status().read().bits(),
            cracen.pk().status().read().errorflags().bits(),
            cracen.pk().status().read().pkbusy().bit_is_set(),
            cracen.pk().status().read().intrptstatus().bit_is_set(),
            cracen.pk().status().read().failptr().bits(),
        );
        cortex_m::asm::nop();

        for i in 0..16 {
            let bytes = unsafe { read32_bytes(slot_addr(i)) };
            if bytes != &[0x00; 32][..] {
                info!("Bytes slot {:x} ({:02}): {:02x}", i, i, bytes);
            }
        }

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

#[inline(always)]
fn slot_addr(slot: u32) -> u32 {
    0x5180_8000 + slot * 0x200 + 0x1E0
}
