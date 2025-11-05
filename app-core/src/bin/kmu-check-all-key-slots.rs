#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 RNG example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    let kmu = p.global_kmu_s;

    for slot in 0..32u32 {
        kmu.keyslot().write(|w| unsafe { w.bits(slot) });

        // Clear stale events
        kmu.events_metadataread()
            .write(|w| w.events_metadataread().clear_bit());
        kmu.events_revoked()
            .write(|w| w.events_revoked().clear_bit());
        kmu.events_error().write(|w| w.events_error().clear_bit());

        // Trigger metadata read
        kmu.tasks_readmetadata()
            .write(|w| w.tasks_readmetadata().set_bit());

        // Wait for completion
        while kmu.events_metadataread().read().bits() == 0
            && kmu.events_error().read().bits() == 0
            && kmu.events_revoked().read().bits() == 0
        {
            cortex_m::asm::nop();
        }

        // Decode result
        if kmu.events_metadataread().read().bits() != 0 {
            let md = kmu.metadata().read().bits();
            kmu.events_metadataread()
                .write(|w| w.events_metadataread().clear_bit());
            if md == 0xFFFFFFFF {
                info!("Slot {:02}: ERASED (metadata = 0xFFFFFFFF)", slot);
            } else {
                info!("Slot {:02}: PROVISIONED, metadata = 0x{:08X}", slot, md);
            }
        } else if kmu.events_revoked().read().bits() != 0 {
            info!("Slot {:02}: REVOKED", slot);
            kmu.events_revoked()
                .write(|w| w.events_revoked().clear_bit());
        } else if kmu.events_error().read().bits() != 0 {
            info!("Slot {:02}: ERROR / EMPTY", slot);
            kmu.events_error().write(|w| w.events_error().clear_bit());
        }
    }

    loop {
        cortex_m::asm::nop();
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PsaKeyPolicy {
    pub usage: u32,
    pub alg: u32,
    pub alg2: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MbedtlsSvcKeyId {
    pub key_id: u32,
    pub owner: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PsaKeyAttributes {
    pub type_: u32,
    pub bits: u32,
    pub lifetime: u32,
    pub policy: PsaKeyPolicy,
    pub id: MbedtlsSvcKeyId,
}
