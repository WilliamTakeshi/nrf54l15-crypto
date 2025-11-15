#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac as _;
use panic_probe as _;

use p256::{
    SecretKey,
    ecdsa::{Signature, SigningKey, signature::Signer, signature::Verifier},
};

#[entry]
fn main() -> ! {
    info!("ECDSA no_std signing example...");

    let msg = b"example";

    let p = nrf54l15_app_pac::Peripherals::take().unwrap();
    p.global_p2_s.pin_cnf(8).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(10).write(|w| w.dir().output());
    p.global_p2_s.pin_cnf(7).write(|w| w.dir().output());
    p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
    p.global_p2_s.outclr().write(|w| w.pin10().bit(true));
    let priv_key_bytes: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x02,
    ];
    // Start
    loop {
        p.global_p2_s.outset().write(|w| w.pin8().bit(true));
        p.global_p2_s.outset().write(|w| w.pin10().bit(true));

        let secret = SecretKey::from_slice(&priv_key_bytes).unwrap();
        let signing_key = SigningKey::from(&secret);

        let sig: Signature = signing_key.sign(&msg.as_slice());

        // Finish
        p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
        p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

        // info!("hash = {:02x}", hash.as_slice());
        info!("signatureraw = {:02x}", sig.to_bytes().as_slice());
        p.global_p2_s.outset().write(|w| w.pin8().bit(true));
        p.global_p2_s.outset().write(|w| w.pin10().bit(true));

        // Derive verify key from signing key
        let verify_key = signing_key.verifying_key();

        // Verify the signature
        match verify_key.verify(msg.as_slice(), &sig) {
            Ok(()) => info!("Signature verified successfully"),
            Err(_) => info!("Signature verification failed"),
        }
        p.global_p2_s.outclr().write(|w| w.pin8().bit(true));
        p.global_p2_s.outclr().write(|w| w.pin10().bit(true));

        for _ in 0..200_000 {
            cortex_m::asm::nop();
        }
    }

    // loop {
    //     cortex_m::asm::nop();
    // }
}
