#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

// const BUF_SIZE: usize = 128;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 CryptoMaster SHA-256 example...");

    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    let mut output_buf: [u8; 32] = [0x00; 32];
    let output_buf_ptr = output_buf.as_mut_ptr();

    // Last descriptor
    let last_descriptor: *mut SxDesc = 1 as *mut SxDesc;

    // Outer descriptor (top-level m)
    let mut output_outer = SxDesc {
        addr: output_buf_ptr, // <(OUTPUT)>
        next: last_descriptor,
        sz: 536870944, // 0x20000010
        dmatag: 32,
    };

    // Create the actual string in memory
    static mut EXAMPLE_STR: [u8; 17] = *b"Example string123";
    let mut bytes_0806: [u8; 4] = [0x08, 0x06, 0x00, 0x00];
    let addr_0806 = bytes_0806.as_mut_ptr();

    // Middle descriptor
    let mut input_mid = SxDesc {
        addr: core::ptr::addr_of_mut!(EXAMPLE_STR) as *mut u8, // "Example string"
        next: last_descriptor,
        // sz: 536870928, // 0x20000010 == 16
        sz: 536870932, // 0x20000014 == 20 (after EXAMPLE_STR sz 17)
        // dmatag: 547, // *b"Example string\0";
        // dmatag: 291, //*b"Example string1\0";
        // dmatag: 35, //*b"Example string12";
        dmatag: 803, //*b"Example string123";
    };

    // Outer descriptor (m)
    let mut input_outer = SxDesc {
        addr: addr_0806,
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
        let bytes = core::slice::from_raw_parts(output_buf_ptr, 32);
        info!("output bytes: {:02x}", bytes);
    }

    loop {
        cortex_m::asm::nop();
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct SxDesc {
    pub addr: *mut u8,
    pub next: *mut SxDesc,
    pub sz: u32,
    pub dmatag: u32,
}
