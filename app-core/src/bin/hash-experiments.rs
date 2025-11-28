#![no_std]
#![no_main]

use app_core::{SxDesc, dmatag_for, sz};
use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

// "3fb0a766aa053d3faef820bb660349bf2d9c86d6fa2e6f6a6ffc5d7c216a7687"
// "162f359ff2c722fd7b912c06dc570c1553d03f8eab52e52a8ff07e67faf70f81"

fn first_round() -> ([u8; 32], nrf54l15_app_pac::GlobalCracencoreS) {
    info!("Starting nRF54L15 CryptoMaster SHA example...");
    // FIRST ROUND
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    let dma = p.global_cracencore_s.cryptmstrdma();

    // let input: &[u8; 64] = b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let input: &[u8; 64] = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let mut state: [u8; 32] = [0x00; 32];
    let state_ptr = state.as_mut_ptr();

    // 4-byte algorithm header
    let header: [u8; 4] = [0x08, 0x00, 0x00, 0x00];

    #[allow(
        clippy::manual_dangling_ptr,
        reason = "nRF54L15 needs this pointer to be on address 1"
    )]
    let last_desc: *mut SxDesc = 1 as *mut SxDesc;

    let mut out_desc = SxDesc {
        addr: state_ptr,
        next: last_desc,
        sz: 536870944,
        dmatag: 32,
    };

    // let mut some_desc = SxDesc {
    //     addr: input2.as_ptr() as *mut u8,
    //     next: last_desc,
    //     sz: 536870913,
    //     dmatag: 35,
    //     // sz: sz(input.len()),
    //     // dmatag: dmatag_for(input.len()),
    // };

    // let mut data_desc = SxDesc {
    //     addr: input.as_ptr() as *mut u8,
    //     next: &mut some_desc,
    //     sz: 63,
    //     dmatag: 3,
    //     // sz: sz(input.len()),
    //     // dmatag: dmatag_for(input.len()),
    // };

    let mut data_desc = SxDesc {
        addr: input.as_ptr() as *mut u8,
        next: last_desc,
        sz: sz(input.len()),
        dmatag: dmatag_for(input.len()),
    };

    let mut in_desc = SxDesc {
        addr: header.as_ptr() as *mut u8,
        next: &mut data_desc,
        sz: sz(4),
        dmatag: 19,
    };

    p.global_cracen_s.enable().write(|w| {
        w.cryptomaster().set_bit();
        w.rng().set_bit();
        w.pkeikg().set_bit()
    });

    unsafe { app_core::load_microcode() };

    dma.fetchaddrlsb()
        .write(|w| unsafe { w.bits((&mut in_desc) as *mut _ as u32) });

    dma.pushaddrlsb()
        .write(|w| unsafe { w.bits((&mut out_desc) as *mut _ as u32) });

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

    info!("state: {:02x}", state);

    (state, p.global_cracencore_s)
}

use core::mem;

pub fn print_var_info<T>(name: &str, var: &T) {
    let addr = var as *const T as usize;
    let size = mem::size_of::<T>();
    let aligned32 = (addr & 0x1F) == 0;
    let aligned16 = (addr & 0x0F) == 0;

    defmt::info!(
        "print_var_info {} @ 0x{:08x}, size = {} bytes, aligned32 = {}, aligned16 = {}",
        name,
        addr,
        size,
        aligned32,
        aligned16,
    );
}

fn second_round(state: [u8; 32], cracencore_s: nrf54l15_app_pac::GlobalCracencoreS) -> () {
    info!("Starting nRF54L15 CryptoMaster SHA example...");
    // SECOND ROUND
    let dma = cracencore_s.cryptmstrdma();

    let input: &[u8; 64] = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\0\0";

    let pad: [u8; 66] = [
        0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x03, 0xf0,
    ];

    let mut out: [u8; 32] = [0x00; 32];
    let out_ptr = out.as_mut_ptr();

    // 4-byte algorithm header
    let header: [u8; 4] = [0x08, 0x04, 0x00, 0x00];

    #[allow(
        clippy::manual_dangling_ptr,
        reason = "nRF54L15 needs this pointer to be on address 1"
    )]
    let last_desc: *mut SxDesc = 1 as *mut SxDesc;

    let mut out_desc = SxDesc {
        addr: out_ptr,
        next: last_desc,
        sz: sz(32),
        dmatag: 32,
    };

    let mut some_desc = SxDesc {
        addr: pad.as_ptr() as *mut u8, //pad
        next: last_desc,
        sz: 0x2000_0000 | 66, // pad
        dmatag: 35,
    };

    let mut data_desc = SxDesc {
        addr: input.as_ptr() as *mut u8,
        next: &mut some_desc,
        sz: 62,
        dmatag: 3,
    };

    let mut state_desc = SxDesc {
        addr: state.as_ptr() as *mut u8,
        next: &mut data_desc,
        sz: sz(32),
        dmatag: 99,
    };

    let mut in_desc = SxDesc {
        addr: header.as_ptr() as *mut u8,
        next: &mut state_desc,
        sz: sz(4),
        dmatag: 19,
    };

    dma.fetchaddrlsb()
        .write(|w| unsafe { w.bits((&mut in_desc) as *mut _ as u32) });

    dma.pushaddrlsb()
        .write(|w| unsafe { w.bits((&mut out_desc) as *mut _ as u32) });

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

    info!("out: {:02x}", out);
}

#[entry]
fn main() -> ! {
    let (state, cracencore_s) = first_round();

    second_round(state.clone(), cracencore_s);

    loop {
        cortex_m::asm::nop();
    }
}
