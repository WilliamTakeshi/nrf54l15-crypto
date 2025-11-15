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
    pub attr_and_len: [u8; 4], // must be 11 + len
}

#[repr(C)]
struct JobList([EcbJob; 2]);

impl EcbJob {
    // Check nRF54L15 datasheet
    // 8.6.2 EasyDMA
    // The scatter-gather functionality allows EasyDMA to collect data from multiple memory regions, instead of
    // one contigous block. The memory regions are described by a job list. The job list consists of one or more
    // job entries that consist of a 32-bit address field, 8-bit attribute field, and 24-bit length field.
    // The attribute field must be set to 11.
    pub fn new(ptr: *const u8, length: u8) -> Self {
        EcbJob {
            ptr: ptr as u32,
            attr_and_len: [length, 0, 0, 11],
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
    info!("Starting nRF54L15 AES-ECB example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();
    let ecb = p.global_ecb00_s;

    // Key: 4C68384139F574D836BCF34E9DFB01BF
    // Plaintext: 0213243546576879acbdcedfe0f10213
    // Expected Encrypted Output: 99ad1b5226a37e3e058e3b8e27c2c666
    let input_buf: [u8; 16] = [
        0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, 0x02,
        0x13,
    ];

    let in_ptr = input_buf.as_ptr();

    let mut input_jobs = JobList([EcbJob::new(in_ptr, 16), EcbJob::zero()]);

    let input_jobs_ptr = core::ptr::addr_of_mut!(input_jobs) as *mut u8;

    let mut output_buf: [u8; 16] = [0x00; 16];

    let out_ptr = core::ptr::addr_of_mut!(output_buf) as *mut u8;

    let mut output_jobs: JobList = JobList([EcbJob::new(out_ptr, 16), EcbJob::zero()]);
    let output_jobs_ptr = core::ptr::addr_of_mut!(output_jobs) as *mut u8;

    p.global_p2_s.pin_cnf(8).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(10).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(7).write(|w| w.dir().output());
    p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
    p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

    loop {
        p.global_p2_s.outset().write(|w| w.pin8().bit(true));
        p.global_p2_s.outset().write(|w| w.pin10().bit(true));

        //The KEY.VALUE registers are populated as follows:
        // • KEY.VALUE[0] = 0x9DFB01BF
        // • KEY.VALUE[1] = 0x36BCF34E
        // • KEY.VALUE[2] = 0x39F574D8
        // • KEY.VALUE[3] = 0x4C683841
        ecb.key()
            .value(0)
            .write(|w| unsafe { w.value().bits(0x9DFB01BF) });
        ecb.key()
            .value(1)
            .write(|w| unsafe { w.value().bits(0x36BCF34E) });
        ecb.key()
            .value(2)
            .write(|w| unsafe { w.value().bits(0x39F574D8) });
        ecb.key()
            .value(3)
            .write(|w| unsafe { w.value().bits(0x4C683841) });

        ecb.in_()
            .ptr()
            .write(|w| unsafe { w.ptr().bits(input_jobs_ptr as u32) });
        ecb.out()
            .ptr()
            .write(|w| unsafe { w.ptr().bits(output_jobs_ptr as u32) });
        ecb.tasks_start().write(|w| w.tasks_start().trigger());

        while ecb.events_end().read().bits() == 0 {
            let end = ecb.events_end().read().bits();
            let err = ecb.events_error().read().bits();

            if err != 0 {
                info!("END={}, ERROR={}", end, err);
            }
        }

        p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
        p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

        info!("Done");
        info!("output_buf: {:02x}", output_buf);

        for _ in 0..200_000 {
            cortex_m::asm::nop();
        }
    }

    // loop {
    //     cortex_m::asm::nop();
    // }
}
