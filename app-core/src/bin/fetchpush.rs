#![no_std]
#![no_main]

// use core::ptr;
use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

#[entry]
fn main() -> ! {
    // 1. Source buffer in system RAM filled with 0xFF
    let src_buf: [u8; 32] = [0xFF; 32];
    let src_ptr = src_buf.as_ptr();

    // 2. Empty destination buffer for final push
    let mut dst_buf: [u8; 32] = [0x00; 32];
    let dst_ptr = dst_buf.as_mut_ptr();

    // 3. PK slot address (slot 0)
    let slot_index = 0x0;
    let pk_slot_addr = PK_RAM_BASE + slot_index * SLOT_SIZE;

    // let dma = nrf54l15_app_pac::global_cracencore_s::;

    let src_addr = src as u32;
    // Source: system RAM
    dma.fetchaddrlsb()
        .write(|w| unsafe { w.fetchaddrlsb().bits(src_addr) });

    // Program length and flags.
    dma.fetchlen().write(|w| {
        // FETCHLEN[27:0] number of bytes.
        unsafe { w.fetchlen().bits(byte_len as u32) }
            // 0 = use address from FETCHADDRLSB
            .fetchcstaddr()
            .clear_bit()
            // 0 = no realign
            .fetchrealign()
            .clear_bit()
            // 0 = zero padding disabled
            .fetchzpadding()
            .clear_bit()
    });

    dma.start().write(|w| w.startfetch().set_bit());

    while dma.status().read().fetchbusy().bit_is_set() {}

    // 5. Push from CRACEN internal RAM → system RAM
    dma.pushaddrlsb()
        .write(|w| unsafe { w.pushaddrlsb().bits(dst_addr) });
    dma.pushlen().write(|w| {
        unsafe { w.pushlen().bits(byte_len as u32) }
            .pushcstaddr()
            .clear_bit()
            .pushrealign()
            .clear_bit()
            // PUSHDISCARD=0 means data is actually written to dst
            .pushdiscard()
            .clear_bit()
    });

    dma.start().write(|w| w.startpush().set_bit());

    while dma.status().read().pushbusy().bit_is_set() {}

    // 6. Confirm content
    for (i, b) in dst_buf.iter().enumerate() {
        assert_eq!(*b, 0xFF, "byte {} mismatch", i);
    }

    loop {}
}
