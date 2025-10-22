#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

// SHA-256 produces 256-bit (32-byte) output
const SHA256_DIGEST_SIZE: usize = 32;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 HASH (SHA-256) example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    // 1. Enable HASH peripheral in CRACEN
    p.global_cracen_s
        .enable()
        .write(|w| w.cryptomaster().set_bit());

    // Test message: "Hello, nRF54L15!"
    let message: &[u8] = b"Hello, nRF54L15!";

    info!(
        "Computing SHA-256 hash of: '{}'",
        core::str::from_utf8(message).unwrap()
    );

    // Buffer for the hash output (32 bytes for SHA-256)
    let mut hash_output = [0u32; 8]; // 8 x 32-bit words = 256 bits

    // Compute the hash
    compute_sha256(&p, message, &mut hash_output);

    // Display the hash result
    info!("SHA-256 Hash:");
    for (i, word) in hash_output.iter().enumerate() {
        info!("  Word {}: 0x{:08x}", i, word);
    }

    info!("\nHash computation complete!");

    loop {
        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }
}

fn compute_sha256(p: &nrf54l15_app_pac::Peripherals, data: &[u8], output: &mut [u32; 8]) {
    // The CRACEN cryptographic engine uses DMA for data transfer
    // We need to set up fetch (input) and push (output) DMA operations

    let input_addr = data.as_ptr() as u32;
    let output_addr = output.as_mut_ptr() as u32;

    // Configure DMA Fetch (input data)
    // Split 32-bit address into LSB and MSB
    p.global_cracencore_s
        .cryptmstrdma()
        .fetchaddrlsb()
        .write(|w| unsafe { w.bits(input_addr) });

    p.global_cracencore_s
        .cryptmstrdma()
        .fetchaddrmsb()
        .write(|w| unsafe { w.bits(0) }); // Upper bits for 64-bit addressing (0 for 32-bit systems)

    // Set fetch length (number of bytes to hash)
    p.global_cracencore_s
        .cryptmstrdma()
        .fetchlen()
        .write(|w| unsafe { w.bits(data.len() as u32) });

    // Configure DMA Push (output digest)
    p.global_cracencore_s
        .cryptmstrdma()
        .pushaddrlsb()
        .write(|w| unsafe { w.bits(output_addr) });

    p.global_cracencore_s
        .cryptmstrdma()
        .pushaddrmsb()
        .write(|w| unsafe { w.bits(0) });

    // Set push length (32 bytes for SHA-256 digest)
    p.global_cracencore_s
        .cryptmstrdma()
        .pushlen()
        .write(|w| unsafe { w.bits(SHA256_DIGEST_SIZE as u32) });

    // Configure the crypto operation
    // Bit 0: Enable
    // Bit 1-3: Algorithm select (SHA-256)
    p.global_cracencore_s
        .cryptmstrdma()
        .config()
        .write(|w| unsafe { w.bits(0x01) }); // Enable DMA

    // Start the DMA operation
    p.global_cracencore_s
        .cryptmstrdma()
        .start()
        .write(|w| unsafe { w.bits(0x01) });

    // Wait for operation to complete
    // Poll status register until busy bit is clear
    loop {
        let status = p.global_cracencore_s.cryptmstrdma().status().read().bits();

        // Check if operation is complete (bit 0 = busy)
        if (status & 0x01) == 0 {
            break;
        }

        cortex_m::asm::nop();
    }

    // The hash output has been written to the output buffer via DMA
}
