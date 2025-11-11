#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
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

    // TODO: shrink this unsafe
    unsafe {
        let data3: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];

        write_block(0x518091e0, &data3); // Block 8

        // 5ECBE4D1A6330A44C8F7EF951D4BF165E6C6B721EFADA985FB41661BC6E7FD6C
        let data2: [u8; 32] = [
            0x5E, 0xCB, 0xE4, 0xD1, 0xA6, 0x33, 0x0A, 0x44, 0xC8, 0xF7, 0xEF, 0x95, 0x1D, 0x4B,
            0xF1, 0x65, 0xE6, 0xC6, 0xB7, 0x21, 0xEF, 0xAD, 0xA9, 0x85, 0xFB, 0x41, 0x66, 0x1B,
            0xC6, 0xE7, 0xFD, 0x6C,
        ];

        write_block(0x518099e0, &data2); // Block 12

        let data: [u8; 32] = [
            0x87, 0x34, 0x64, 0x0C, 0x49, 0x98, 0xFF, 0x7E, 0x37, 0x4B, 0x06, 0xCE, 0x1A, 0x64,
            0xA2, 0xEC, 0xD8, 0x2A, 0xB0, 0x36, 0x38, 0x4F, 0xB8, 0x3D, 0x9A, 0x79, 0xB1, 0x27,
            0xA2, 0x7D, 0x50, 0x32,
        ];
        write_block(0x51809be0, &data); // Block 13

        cracen.pk().pointers().write(|w| {
            w.opptra().bits(12);
            w.opptrb().bits(8);
            w.opptrc().bits(10)
        });

        cracen.pk().command().write(|w| w.bits(0x10101F22)); // 269492002

        cracen.pk().control().write(|w| {
            w.start().set_bit();
            w.clearirq().set_bit()
        });
    }

    info!("Done");

    loop {
        let status = cracen.pk().status().read().bits();
        info!("Status: {:b}", status);
        cortex_m::asm::nop();

        let bytes = unsafe { read32_bytes(0x5180_95e0) };
        info!("Bytes: {:x}", bytes);

        let bytes = unsafe { read32_bytes(0x518097e0) };
        info!("Bytes: {:x}", bytes);

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
