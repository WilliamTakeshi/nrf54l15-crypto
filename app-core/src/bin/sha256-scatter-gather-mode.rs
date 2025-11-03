#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

// Supported hash algorithm bitmasks
#[allow(dead_code)]
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum HashAlg {
    // Sha1 = 0x02, // TODO: implement
    Sha2_224 = 0x04,
    Sha2_256 = 0x08,
    Sha2_384 = 0x10,
    Sha2_512 = 0x20,
    // Sm3 = 0x40,      // TODO: implement
}

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 CryptoMaster SHA-256 example...");
    const OUTPUT_BUF_LEN: usize = 28;
    const HASH_ALGO: u8 = HashAlg::Sha2_224 as u8;
    const INPUT_BUF_LEN: usize = 14;
    static mut INPUT: [u8; INPUT_BUF_LEN] = *b"Example string";

    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    let mut output_buf: [u8; OUTPUT_BUF_LEN] = [0x00; OUTPUT_BUF_LEN];
    let output_buf_ptr = output_buf.as_mut_ptr();

    let mut bytes_hash: [u8; 4] = [HASH_ALGO, 0x06, 0x00, 0x00];
    let addr_hash = bytes_hash.as_mut_ptr();

    let dmatag = dmatag_for(INPUT_BUF_LEN);
    let sz = sz(INPUT_BUF_LEN);

    // Last descriptor
    #[allow(
        clippy::manual_dangling_ptr,
        reason = "nRF54L15 needs this pointer to be on address 1"
    )]
    let last_descriptor: *mut SxDesc = 1 as *mut SxDesc;

    // Outer descriptor (output)
    let mut output_outer = SxDesc {
        addr: output_buf_ptr,
        next: last_descriptor,
        sz: (0x20000000 + OUTPUT_BUF_LEN as u32),
        dmatag: 32,
    };

    // Middle descriptor (input)
    let mut input_mid = SxDesc {
        addr: core::ptr::addr_of_mut!(INPUT) as *mut u8, // "Example string"
        next: last_descriptor,
        sz,
        dmatag,
    };

    // Outer descriptor (input)
    let mut input_outer = SxDesc {
        addr: addr_hash,
        next: &mut input_mid as *mut SxDesc,
        sz: 536870916, // 0x20000004
        dmatag: 19,
    };

    // Enable CryptoMaster
    p.global_cracen_s.enable().write(|w| {
        w.cryptomaster().set_bit();
        w.rng().set_bit();
        w.pkeikg().set_bit()
    });

    let dma = p.global_cracencore_s.cryptmstrdma();

    let output_outer_ptr = &mut output_outer as *mut SxDesc;
    let input_outer_ptr = &mut input_outer as *mut SxDesc;

    // 1. Configure DMA fetch (input)
    dma.fetchaddrlsb()
        .write(|w| unsafe { w.bits(input_outer_ptr as u32) });

    // 2. Configure DMA push (output)
    dma.pushaddrlsb()
        .write(|w| unsafe { w.bits(output_outer_ptr as u32) });

    // 3. Configure DMA
    dma.config().write(|w| {
        w.fetchctrlindirect().set_bit();
        w.pushctrlindirect().set_bit();
        w.fetchstop().clear_bit();
        w.pushstop().clear_bit();
        w.softrst().clear_bit()
    });

    dma.start().write(|w| {
        w.startfetch().set_bit();
        w.startpush().set_bit()
    });

    while dma.status().read().fetchbusy().bit_is_set() {}
    while dma.status().read().pushbusy().bit_is_set() {}

    // output
    unsafe {
        let bytes = core::slice::from_raw_parts(output_buf_ptr, output_buf.len());
        info!("output bytes: {:02x}", bytes);
    }

    loop {
        cortex_m::asm::nop();
    }
}

fn dmatag_for(input: usize) -> u32 {
    const TAG_BASE: u32 = 0x23;
    const TAG_0: u32 = 0x000;
    const TAG_1: u32 = 0x300;
    const TAG_2: u32 = 0x200;
    const TAG_3: u32 = 0x100;

    match input % 4 {
        0 => TAG_BASE | TAG_0, // -> 0x023 = 35
        1 => TAG_BASE | TAG_1, // -> 0x323 = 803
        2 => TAG_BASE | TAG_2, // -> 0x223 = 547
        3 => TAG_BASE | TAG_3, // -> 0x123 = 291
        _ => panic!("impossible state"),
    }
}

fn sz(n: usize) -> u32 {
    let group_end = ((n - 1) / 4 + 1) * 4;
    (group_end | 0x2000_0000).try_into().unwrap()
}

#[repr(C)]
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct SxDesc {
    pub addr: *mut u8,
    pub next: *mut SxDesc,
    pub sz: u32,
    pub dmatag: u32,
}
