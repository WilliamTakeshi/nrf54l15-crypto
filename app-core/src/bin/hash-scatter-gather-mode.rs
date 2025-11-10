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
    Sha1 = 0x02,
    Sha2_224 = 0x04,
    Sha2_256 = 0x08,
    Sha2_384 = 0x10,
    Sha2_512 = 0x20,
    // Sm3 = 0x40,      // TODO: implement
}

#[derive(Debug)]
pub enum ShaError {
    Busy,
    InvalidInput,
}

const fn hash_out_len(algo: HashAlg) -> usize {
    match algo {
        HashAlg::Sha1 => 20,
        HashAlg::Sha2_224 => 28,
        HashAlg::Sha2_256 => 32,
        HashAlg::Sha2_384 => 48,
        HashAlg::Sha2_512 => 64,
    }
}

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 CryptoMaster SHA example...");

    let input = b"example";
    info!("input: {:02x}", input);

    let mut out_sha1 = [0u8; 20];
    cracen_sha1(input, &mut out_sha1).unwrap();
    info!("output bytes SHA1: {:02x}", out_sha1);

    let mut out_sha224 = [0u8; 28];
    cracen_sha224(input, &mut out_sha224).unwrap();
    info!("output bytes SHA2_224: {:02x}", out_sha224);

    let mut out_sha256 = [0u8; 32];
    cracen_sha256(input, &mut out_sha256).unwrap();
    info!("output bytes SHA2_256: {:02x}", out_sha256);

    let mut out_sha384 = [0u8; 48];
    cracen_sha384(input, &mut out_sha384).unwrap();
    info!("output bytes SHA2_384: {:02x}", out_sha384);

    let mut out_sha512 = [0u8; 64];
    cracen_sha512(input, &mut out_sha512).unwrap();
    info!("output bytes SHA2_512: {:02x}", out_sha512);

    loop {
        cortex_m::asm::nop();
    }
}

pub fn cracen_sha1(input: &[u8], out: &mut [u8; 20]) -> Result<(), ShaError> {
    cracen_hash(input, out, HashAlg::Sha1)
}

pub fn cracen_sha224(input: &[u8], out: &mut [u8; 28]) -> Result<(), ShaError> {
    cracen_hash(input, out, HashAlg::Sha2_224)
}

pub fn cracen_sha256(input: &[u8], out: &mut [u8; 32]) -> Result<(), ShaError> {
    cracen_hash(input, out, HashAlg::Sha2_256)
}

pub fn cracen_sha384(input: &[u8], out: &mut [u8; 48]) -> Result<(), ShaError> {
    cracen_hash(input, out, HashAlg::Sha2_384)
}

pub fn cracen_sha512(input: &[u8], out: &mut [u8; 64]) -> Result<(), ShaError> {
    cracen_hash(input, out, HashAlg::Sha2_512)
}

fn cracen_hash<const N: usize>(
    input: &[u8],
    out: &mut [u8; N],
    alg: HashAlg,
) -> Result<(), ShaError> {
    if N != hash_out_len(alg) {
        return Err(ShaError::InvalidInput);
    }
    if input.is_empty() {
        return Err(ShaError::InvalidInput);
    }

    // TODO: Steal isn't the right way of doing it.
    // In production you should take once and pass the reference
    let p = unsafe { nrf54l15_app_pac::Peripherals::steal() };

    let dma = p.global_cracencore_s.cryptmstrdma();

    let out_ptr = out.as_mut_ptr();

    // 4-byte algorithm header
    let mut header = [alg as u8, 0x06, 0x00, 0x00];

    // Last descriptor (address = 1)
    #[allow(
        clippy::manual_dangling_ptr,
        reason = "nRF54L15 needs this pointer to be on address 1"
    )]
    let last_desc: *mut SxDesc = 1 as *mut SxDesc;

    // Output descriptor
    let mut out_desc = SxDesc {
        addr: out_ptr,
        next: last_desc,
        sz: sz(N),
        dmatag: 32,
    };

    // Middle descriptor (input)
    let mut mid_desc = SxDesc {
        addr: input.as_ptr() as *mut u8,
        next: last_desc,
        sz: sz(input.len()),
        dmatag: dmatag_for(input.len()),
    };

    // Outer descriptor (input)
    let mut in_desc = SxDesc {
        addr: header.as_mut_ptr(),
        next: &mut mid_desc,
        sz: sz(4),
        dmatag: 19,
    };

    // Enable cryptomaster
    p.global_cracen_s.enable().write(|w| {
        w.cryptomaster().set_bit();
        w.rng().set_bit();
        w.pkeikg().set_bit()
    });

    // Configure DMA source
    dma.fetchaddrlsb()
        .write(|w| unsafe { w.bits((&mut in_desc) as *mut _ as u32) });

    // Configure DMA sink
    dma.pushaddrlsb()
        .write(|w| unsafe { w.bits((&mut out_desc) as *mut _ as u32) });

    dma.config().write(|w| {
        w.fetchctrlindirect().set_bit();
        w.pushctrlindirect().set_bit();
        w.fetchstop().clear_bit();
        w.pushstop().clear_bit();
        w.softrst().clear_bit()
    });

    // Start DMA
    dma.start().write(|w| {
        w.startfetch().set_bit();
        w.startpush().set_bit()
    });

    // Wait
    while dma.status().read().fetchbusy().bit_is_set() {}
    while dma.status().read().pushbusy().bit_is_set() {}

    Ok(())
}

// TODO: Remove magic numbers
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
    const DMA_REALIGN: usize = 0x2000_0000;
    let group_end = ((n - 1) / 4 + 1) * 4;
    (group_end | DMA_REALIGN).try_into().unwrap()
}

#[repr(C)]
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct SxDesc {
    pub addr: *mut u8,
    pub next: *mut SxDesc,
    pub sz: u32,
    pub dmatag: u32,
}
