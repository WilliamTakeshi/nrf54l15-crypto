#![no_std]

// Supported hash algorithm bitmasks
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

pub fn cracen_sha1(
    p: &nrf54l15_app_pac::Peripherals,
    input: &[u8],
    out: &mut [u8; 20],
) -> Result<(), ShaError> {
    cracen_hash(p, input, out, HashAlg::Sha1)
}

pub fn cracen_sha224(
    p: &nrf54l15_app_pac::Peripherals,
    input: &[u8],
    out: &mut [u8; 28],
) -> Result<(), ShaError> {
    cracen_hash(p, input, out, HashAlg::Sha2_224)
}

pub fn cracen_sha256(
    p: &nrf54l15_app_pac::Peripherals,
    input: &[u8],
    out: &mut [u8; 32],
) -> Result<(), ShaError> {
    cracen_hash(p, input, out, HashAlg::Sha2_256)
}

pub fn cracen_sha384(
    p: &nrf54l15_app_pac::Peripherals,
    input: &[u8],
    out: &mut [u8; 48],
) -> Result<(), ShaError> {
    cracen_hash(p, input, out, HashAlg::Sha2_384)
}

pub fn cracen_sha512(
    p: &nrf54l15_app_pac::Peripherals,
    input: &[u8],
    out: &mut [u8; 64],
) -> Result<(), ShaError> {
    cracen_hash(p, input, out, HashAlg::Sha2_512)
}

fn cracen_hash<const N: usize>(
    p: &nrf54l15_app_pac::Peripherals,
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
    (group_end | DMA_REALIGN) as u32
}

#[repr(C)]
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct SxDesc {
    pub addr: *mut u8,
    pub next: *mut SxDesc,
    pub sz: u32,
    pub dmatag: u32,
}

// TODO: make this generic over the hash function
pub fn cracen_hmac_sha256(
    p: &nrf54l15_app_pac::Peripherals,
    key: &[u8],
    message: &[u8],
    out: &mut [u8; 32],
) -> Result<(), ShaError> {
    // ---- 1. Normalize key ----
    const BLOCK: usize = 64;
    let mut key_block = [0u8; BLOCK];

    if key.len() > BLOCK {
        // K = H(K)
        let mut tmp = [0u8; 32];
        cracen_sha256(p, key, &mut tmp)?;
        key_block[..32].copy_from_slice(&tmp);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    // ---- 2. ipad / opad ----
    let mut ipad = [0u8; BLOCK];
    let mut opad = [0u8; BLOCK];

    for i in 0..BLOCK {
        let b = key_block[i];
        ipad[i] = b ^ 0x36;
        opad[i] = b ^ 0x5c;
    }

    // ---- 3. inner hash = SHA256(ipad || message) ----
    let mut inner_buf = [0u8; BLOCK + 1024]; // static buffer, adjust if needed
    let mut inner_len = 0;

    inner_buf[..BLOCK].copy_from_slice(&ipad);
    inner_len += BLOCK;

    inner_buf[inner_len..inner_len + message.len()].copy_from_slice(message);
    inner_len += message.len();

    let mut inner_hash = [0u8; 32];
    cracen_sha256(p, &inner_buf[..inner_len], &mut inner_hash)?;

    // ---- 4. outer hash = SHA256(opad || inner_hash) ----
    let mut outer_buf = [0u8; BLOCK + 32];
    outer_buf[..BLOCK].copy_from_slice(&opad);
    outer_buf[BLOCK..BLOCK + 32].copy_from_slice(&inner_hash);

    cracen_sha256(p, &outer_buf, out)?;

    Ok(())
}

pub fn rng(p: &nrf54l15_app_pac::Peripherals, buf: &mut [u8]) {
    // TODO: check if we really need this for every try,
    // maybe add a cracen setup before everything.
    p.global_cracen_s.enable().write(|w| w.rng().set_bit());

    p.global_cracencore_s
        .rngcontrol()
        .control()
        .write(|w| w.enable().set_bit());

    let mut idx = 0;

    while idx < buf.len() {
        let level = loop {
            let l = p.global_cracencore_s.rngcontrol().fifolevel().read().bits() as usize;
            if l > 0 {
                break l;
            }
        };

        for fifo_idx in 0..level {
            if idx >= buf.len() {
                break;
            }

            let rnd = p
                .global_cracencore_s
                .rngcontrol()
                .fifo(fifo_idx)
                .read()
                .bits();
            let bytes = rnd.to_le_bytes();

            let remaining = buf.len() - idx;
            let take = remaining.min(4);

            buf[idx..idx + take].copy_from_slice(&bytes[..take]);
            idx += take;

            if idx >= buf.len() {
                break;
            }
        }
    }
}
