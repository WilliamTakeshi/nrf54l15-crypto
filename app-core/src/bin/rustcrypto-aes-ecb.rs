#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac as _;
use panic_probe as _;

use aes::Aes128;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncrypt, KeyInit};

#[entry]
fn main() -> ! {
    // 128-bit AES key
    let key = GenericArray::from([
        0x4C, 0x68, 0x38, 0x41, 0x39, 0xF5, 0x74, 0xD8, 0x36, 0xBC, 0xF3, 0x4E, 0x9D, 0xFB, 0x01,
        0xBF,
    ]);
    // 16-byte plaintext block (AES-ECB always uses 16-byte blocks)
    let mut block = GenericArray::from([
        0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, 0x02,
        0x13,
    ]);

    // Start
    // Initialize AES
    let cipher = Aes128::new(&key);

    // Encrypt *in place*
    cipher.encrypt_block(&mut block);
    // Finish

    info!("AES-128-ECB(plaintext) = {:02x}", block.as_slice());

    loop {
        cortex_m::asm::nop();
    }
}
