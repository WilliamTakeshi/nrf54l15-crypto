#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

static mut HASH_RESULT: [u8; 32] = [0; 32];

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 CryptoMaster SHA-256 example...");

    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    // Enable CryptoMaster
    p.global_cracen_s.enable().write(|w| {
        w.cryptomaster()
            .set_bit()
            .rng()
            .set_bit()
            .pkeikg()
            .set_bit()
    });

    // Example input
    let data: [u8; 64] = [0xFF; 64];

    info!("data len {}", data.len() as u32);

    unsafe {
        let out_ptr = &raw mut HASH_RESULT as u32;
        let out_len = core::mem::size_of::<[u8; 32]>() as u32;

        // 1. Configure DMA fetch (input)
        p.global_cracencore_s
            .cryptmstrdma()
            .fetchaddrlsb()
            .write(|w| w.bits(data.as_ptr() as u32));
        p.global_cracencore_s
            .cryptmstrdma()
            .fetchaddrmsb()
            .write(|w| w.bits(0));
        p.global_cracencore_s
            .cryptmstrdma()
            .fetchlen()
            .write(|w| w.bits(data.len() as u32));

        // 2. Configure DMA push (output)
        p.global_cracencore_s
            .cryptmstrdma()
            .pushaddrlsb()
            .write(|w| w.bits(out_ptr));
        p.global_cracencore_s
            .cryptmstrdma()
            .pushaddrmsb()
            .write(|w| w.bits(0));
        p.global_cracencore_s
            .cryptmstrdma()
            .pushlen()
            .write(|w| w.bits(out_len));

        // 3. Configure CryptoMaster for SHA-256
        p.global_cracencore_s
            .cryptmstrdma()
            .config()
            .write(|w| w.bits(0b00000)); // placeholder for SHA-256

        info!("after configure");
        // 4. Start
        p.global_cracencore_s
            .cryptmstrdma()
            .start()
            .write(|w| w.bits(0b11));

        for _ in 0..200_000 {
            cortex_m::asm::nop();
        }

        info!(
            "p.global_cracencore_s.cryptmstrdma().status().read().bits(): {:b}",
            p.global_cracencore_s.cryptmstrdma().status().read().bits()
        );

        let status = p.global_cracencore_s.cryptmstrdma().status().read().bits() & 0x1;

        info!("status: {:b}", status);

        // 5. Wait complete
        while p.global_cracencore_s.cryptmstrdma().status().read().bits() & 0x1 != 0 {}

        let status = p.global_cracencore_s.cryptmstrdma().status().read().bits();

        info!("data: {:x}", data);
        info!("data: {:x}", data.as_ptr() as u32);

        info!("status: {:b}", status);

        // 6. Read result via raw pointer
        let hash: [u8; 32] = core::ptr::read_volatile(core::ptr::addr_of!(HASH_RESULT));
        info!("result: {:x}", hash);
    }

    loop {
        cortex_m::asm::nop();
    }
}
