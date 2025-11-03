// Demonstrates a DMA round-trip: copy 32 bytes of 0xFF from system RAM into CRACEN slot 0, then push it back into another RAM buffer.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    // 0. Enable CryptoMaster
    p.global_cracen_s.enable().write(|w| {
        w.cryptomaster().set_bit();
        w.rng().set_bit();
        w.pkeikg().set_bit()
    });

    // 1. Source buffer in system RAM filled with 0xFF
    let src_buf: [u8; 32] = [0xFF; 32];
    let src_ptr = src_buf.as_ptr();

    // 2. Empty destination buffer for final push
    let mut dst_buf: [u8; 32] = [0x00; 32];
    let dst_ptr = dst_buf.as_mut_ptr();

    let dma = p.global_cracencore_s.cryptmstrdma();
    // Source: system RAM
    dma.fetchaddrlsb()
        .write(|w| unsafe { w.fetchaddrlsb().bits(src_ptr as u32) });
    // Program length and flags.
    dma.fetchlen().write(|w| {
        unsafe { w.fetchlen().bits(src_buf.len() as u32) }
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
        .write(|w| unsafe { w.pushaddrlsb().bits(dst_ptr as u32) });
    dma.pushlen().write(|w| {
        unsafe { w.pushlen().bits(dst_buf.len() as u32) }
            .pushcstaddr()
            .clear_bit()
            .pushrealign()
            .clear_bit()
            .pushdiscard()
            .clear_bit()
    });

    dma.start().write(|w| w.startpush().set_bit());

    while dma.status().read().pushbusy().bit_is_set() {}

    info!("src_buf: {:02x}", src_buf);
    info!("dst_buf: {:02x}", dst_buf);

    // 6. Confirm content
    for (i, b) in dst_buf.iter().enumerate() {
        assert_eq!(*b, src_buf[i], "byte {} mismatch", i);
    }

    loop {
        cortex_m::asm::nop();
    }
}
