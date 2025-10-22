#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

const BUF_SIZE: usize = 128;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 CryptoMaster SHA-256 example...");

    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    // Enable CryptoMaster
    p.global_cracen_s.enable().write(|w| {
        w.cryptomaster().set_bit();
        w.rng().set_bit();
        w.pkeikg().set_bit()
    });

    // Example input
    let src_buf: [u8; BUF_SIZE] = [0xFF; BUF_SIZE];
    let src_ptr = src_buf.as_ptr();

    // 2. Empty destination buffer for final push
    let mut dst_buf: [u8; BUF_SIZE] = [0x00; BUF_SIZE];
    let dst_ptr = dst_buf.as_mut_ptr();

    let dma = p.global_cracencore_s.cryptmstrdma();
    let pk = p.global_cracencore_s.pk();

    // 3. Configure DMA
    dma.config().write(|w| {
        w.fetchctrlindirect().clear_bit();
        w.pushctrlindirect().clear_bit();
        w.fetchstop().clear_bit();
        w.pushstop().clear_bit();
        w.softrst().clear_bit()
    });

    // 1. Configure DMA fetch (input)
    dma.fetchaddrlsb()
        .write(|w| unsafe { w.bits(src_ptr as u32) });
    dma.fetchlen()
        .write(|w| unsafe { w.bits(src_buf.len() as u32) });

    // 1.1. Start and wait for fetch
    dma.start().write(|w| w.startfetch().set_bit());
    while dma.status().read().fetchbusy().bit_is_set() {}

    pk.pointers().write(|w| {
        unsafe {
            w.opptra().bits(0x1);
            w.opptrb().bits(0x1);
            w.opptrc().bits(0x1);
            w.opptrn().bits(0x1);
        }
        w
    });

    pk.command().write(|w| {
        unsafe {
            // 7-bit OPEADDR
            w.opeaddr().bits(0x01)
        }
    });

    pk.control().write(|w| w.start().set_bit());

    let err = pk.status().read().errorflags().bits();
    if err != 0 {
        let failptr = pk.status().read().failptr().bits() as u8;
        info!("err1: {:b}", err);
        info!("err1: {:x}", err);
        info!("failptr1: {:b}", failptr);
    }
    while pk.status().read().pkbusy().bit_is_set() {}

    if err != 0 {
        let failptr = pk.status().read().failptr().bits() as u8;
        info!("err: {:b}", err);
        info!("err: {:x}", err);
        info!("failptr: {:b}", failptr);
    }

    // 2. Configure DMA push (output)
    dma.pushaddrlsb()
        .write(|w| unsafe { w.bits(dst_ptr as u32) });
    dma.pushlen()
        .write(|w| unsafe { w.bits(dst_buf.len() as u32) });

    // 2.2. Start and wait for fetch
    dma.start().write(|w| w.startpush().set_bit());
    while dma.status().read().pushbusy().bit_is_set() {}

    // info!("dma.status().read(): {:x}", dma.status().read().bits());

    info!("src_buf: {:02x}", src_buf);
    info!("dst_buf: {:02x}", dst_buf);

    loop {
        cortex_m::asm::nop();
    }
}
