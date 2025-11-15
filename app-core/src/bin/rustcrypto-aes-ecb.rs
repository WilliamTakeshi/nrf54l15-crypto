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

    // Finish

    let p = nrf54l15_app_pac::Peripherals::take().unwrap();
    p.global_p2_s.pin_cnf(8).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(10).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(7).write(|w| w.dir().output());
    p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
    p.global_p2_s.outclr().write(|w| w.pin10().bit(true));
    let cipher = Aes128::new(&key);

    loop {
        let mut block = GenericArray::from([
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1,
            0x02, 0x13,
        ]);

        p.global_p2_s.outset().write(|w| w.pin8().bit(true));
        p.global_p2_s.outset().write(|w| w.pin10().bit(true));

        cipher.encrypt_block(&mut block);
        p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
        p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

        // Finish

        info!("AES-128-ECB(plaintext) = {:02x}", block.as_slice());

        for _ in 0..200_000 {
            cortex_m::asm::nop();
        }
    }

    // loop {
    //     cortex_m::asm::nop();
    // }
}
