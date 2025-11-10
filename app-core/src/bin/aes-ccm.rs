#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct EcbJob {
    pub ptr: u32,              // data pointer
    pub attr_and_len: [u8; 4], // length (3 bytes) || attr (1 byte)
}

#[repr(C)]
struct JobListInput([EcbJob; 5]);

#[repr(C)]
struct JobListOutput([EcbJob; 5]);

#[repr(u8)]
pub enum EcbJobAttr {
    Alen = 11,
    Mlen = 12,
    Adata = 13,
    Mdata = 14,
}

impl EcbJob {
    pub fn new(ptr: *const u8, length: u8, tag: EcbJobAttr) -> Self {
        EcbJob {
            ptr: ptr as u32,
            attr_and_len: [length, 0, 0, tag as u8],
        }
    }
    // A job list ends with a zero filled job entry
    pub const fn zero() -> Self {
        EcbJob {
            ptr: 0,
            attr_and_len: [0; 4],
        }
    }
}

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 AES-CCM example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();
    let ccm = p.global_ccm00_s;

    // CONFIG
    ccm.enable().write(|w| w.enable().enabled());

    // For protocols other than Bluetooth, the ADATAMASK register must be set to 0xFF
    // for correct CCM operation; the reset value is configured to support Bluetooth
    ccm.adatamask()
        .write(|w| unsafe { w.adatamask().bits(0xFF) });

    ccm.mode().write(|w| {
        w.mode().encryption();
        w.maclen().m16()
    });

    // Key: 00000000000000000000000000000002
    // Plaintext: 0213243546576879acbdcedfe0f10213
    // Expected Encrypted Output: 99ad1b5226a37e3e058e3b8e27c2c666
    // let input_buf: [u8; 16] = [
    //     0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, 0x02,
    //     0x13,
    // ];

    // INPUT

    let alen_input_buf: [u8; 4] = 13u32.to_le_bytes();
    let alen_input_ptr = alen_input_buf.as_ptr();

    let mlen_input_buf: [u8; 4] = 16u32.to_le_bytes();
    let mlen_input_ptr = mlen_input_buf.as_ptr();

    // let aad_input_buf: [u8; 16] = [0x00; 16];
    // The last 3 are ignored, since we use 13 bytes.
    // But we need 16 bytes to keep everything aligned
    let aad_input_buf: [u8; 16] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00,
    ];
    let aad_input_ptr = aad_input_buf.as_ptr();

    let input_buf: [u8; 16] = [0x00; 16];
    // info!("input_buf: {:02x}", input_buf);
    let in_ptr = input_buf.as_ptr();
    info!("in_ptr: {:02x}", in_ptr);

    let mut input_jobs = JobListInput([
        EcbJob::new(alen_input_ptr, 2, EcbJobAttr::Alen),
        EcbJob::new(mlen_input_ptr, 2, EcbJobAttr::Mlen),
        EcbJob::new(aad_input_ptr, 13, EcbJobAttr::Mdata),
        EcbJob::new(in_ptr, 16, EcbJobAttr::Adata),
        EcbJob::zero(),
    ]);

    let input_jobs_ptr = core::ptr::addr_of_mut!(input_jobs) as *mut u8;

    // OUTPUT
    let alen_output_buf: [u8; 4] = 13u32.to_le_bytes();
    let alen_output_ptr = alen_output_buf.as_ptr();

    let mlen_output_buf: [u8; 4] = 16u32.to_le_bytes();
    let mlen_output_ptr = mlen_output_buf.as_ptr();

    let aad_output_buf: [u8; 16] = [0x00; 16];
    let aad_output_ptr = aad_output_buf.as_ptr();

    let mut output_buf: [u8; 32] = [0x00; 32];
    let out_ptr = core::ptr::addr_of_mut!(output_buf) as *mut u8;

    let mut output_jobs: JobListOutput = JobListOutput([
        EcbJob::new(alen_output_ptr, 2, EcbJobAttr::Alen),
        EcbJob::new(mlen_output_ptr, 2, EcbJobAttr::Mlen),
        EcbJob::new(aad_output_ptr, 13, EcbJobAttr::Adata),
        EcbJob::new(out_ptr, 32, EcbJobAttr::Mdata),
        EcbJob::zero(),
    ]);
    let output_jobs_ptr = core::ptr::addr_of_mut!(output_jobs) as *mut u8;

    //The KEY.VALUE registers are populated as follows:
    // • KEY.VALUE[0] = 0x00000000
    // • KEY.VALUE[1] = 0x00000000
    // • KEY.VALUE[2] = 0x00000000
    // • KEY.VALUE[3] = 0x02000000
    ccm.key()
        .value(0)
        .write(|w| unsafe { w.value().bits(0x00000000) });
    ccm.key()
        .value(1)
        .write(|w| unsafe { w.value().bits(0x00000000) });
    ccm.key()
        .value(2)
        .write(|w| unsafe { w.value().bits(0x00000000) });
    ccm.key()
        .value(3)
        .write(|w| unsafe { w.value().bits(0x00000000) });
    info!("INFO324");

    // Nonce: ""
    ccm.nonce()
        .value(0)
        .write(|w| unsafe { w.value().bits(0xFFFFFFFF) });

    ccm.nonce()
        .value(1)
        .write(|w| unsafe { w.value().bits(0xFFFFFFFF) });

    ccm.nonce()
        .value(2)
        .write(|w| unsafe { w.value().bits(0xFFFFFFFF) });

    ccm.nonce()
        .value(3)
        .write(|w| unsafe { w.value().bits(0x0000023) });

    ccm.in_()
        .ptr()
        .write(|w| unsafe { w.ptr().bits(input_jobs_ptr as u32) });
    ccm.out()
        .ptr()
        .write(|w| unsafe { w.ptr().bits(output_jobs_ptr as u32) });
    ccm.tasks_start().write(|w| w.tasks_start().trigger());

    while ccm.events_end().read().bits() == 0 {
        let end = ccm.events_end().read().bits();
        let err = ccm.events_error().read().bits();

        if err != 0 {
            info!("END={}, ERROR={}", end, err);
        }
    }

    info!("Done");
    info!("output: {:02x}", &output_buf[..16]);
    info!("tag: {:02x}", &output_buf[16..]);

    loop {
        cortex_m::asm::nop();
    }
}
