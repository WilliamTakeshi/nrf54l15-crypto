#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    info!("Starting nRF54L15 KMU example...");
    let p = nrf54l15_app_pac::Peripherals::take().unwrap();

    let kmu = p.global_kmu_s;
    let rramc = p.global_rramc_s;

    // 2. Import Alice + Bob private keys
    pub const _PRIV_ALICE: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x02];

    pub const _PRIV_BOB: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01];

    let _attr = PsaKeyAttributes {
        type_: 28946,
        bits: 256,
        lifetime: 0,
        policy: PsaKeyPolicy {
            usage: 16385,
            alg: 151126016,
            alg2: 0,
        },
        id: MbedtlsSvcKeyId {
            key_id: 0,
            owner: -1006632960,
        },
    };

    const SLOT_ID: u32 = 0;

    // -----Start revoking key slot-----
    kmu.keyslot().write(|w| unsafe { w.bits(SLOT_ID) });
    rramc.config().write(|w| {
        w.wen().set_bit();
        w.writebufsize().unbuffered()
    });
    kmu.tasks_revoke().write(|w| w.tasks_revoke().set_bit());

    // wait until revoked
    while kmu.events_revoked().read().bits() == 0 {
        if kmu.events_error().read().bits() != 0 {
            let es = kmu.events_error().read().bits();
            info!("Revoke error: 0x{:08X}", es);
            kmu.events_error().write(|w| w.events_error().clear_bit());
            break;
        }
    }
    kmu.events_revoked()
        .write(|w| w.events_revoked().clear_bit());
    rramc.config().write(|w| w.wen().clear_bit());
    // -----End revoking key slot-----

    // -----Start provisioning key slot-----
    static mut DEST_BUF: DestData = DestData { value: [0u8; 16] };

    // 1. Asset to store (example secret)
    static mut SRC_DATA: SrcData = SrcData {
        value: [0xAB; 16],
        rpolicy: 0b01u32,
        dest: 0, // fill at runtime
        metadata: 0xffffffff,
    };

    unsafe {
        SRC_DATA.dest = &raw const DEST_BUF as u32;
        let ptr = &raw const SRC_DATA as *const u8;
        info!("ptr: {}", ptr);
        let bytes = core::slice::from_raw_parts(ptr, core::mem::size_of::<SrcData>());

        info!("SRC_DATA bytes ({:02X} bytes): {:02X}", bytes.len(), bytes);
    }

    // static mut SRC_DATA: [u8; 16] = [
    //     0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
    //     0x00,
    // ];

    // 2. Point KMU to our RAM source struct
    let src_addr = &raw const SRC_DATA as u32;
    kmu.src().write(|w| unsafe { w.bits(src_addr) });

    // 3. Select slot
    kmu.keyslot().write(|w| unsafe { w.bits(SLOT_ID) });

    // 4. Enable unbuffered RRAM write
    rramc.config().write(|w| {
        w.wen().set_bit(); // enable write
        w.writebufsize().unbuffered() // unbuffered mode
    });

    kmu.tasks_provision()
        .write(|w| w.tasks_provision().set_bit());

    loop {
        if kmu.events_provisioned().read().bits() != 0 {
            info!("Slot {} provisioned", SLOT_ID);
            kmu.events_provisioned()
                .write(|w| w.events_provisioned().clear_bit());
            break;
        }
        if kmu.events_error().read().bits() != 0 {
            let e = kmu.events_error().read().bits();
            info!("Provisioning error: 0x{:08X}", e);
            kmu.events_error().write(|w| w.events_error().clear_bit());
            break;
        }
    }
    // 6. Disable RRAM write
    rramc.config().write(|w| w.wen().clear_bit());

    info!("Done.");
    // -----End provisioning key slot-----

    // -----Start reading key slot metadata-----
    kmu.keyslot().write(|w| unsafe { w.bits(SLOT_ID) });

    // metadata read
    kmu.tasks_readmetadata()
        .write(|w| w.tasks_readmetadata().set_bit());

    // Wait for completion
    while kmu.events_metadataread().read().bits() == 0 {}

    let md = kmu.metadata().read().bits();

    // Clear the event for next iteration
    kmu.events_metadataread()
        .write(|w| w.events_metadataread().clear_bit());

    info!("KMU slot {:02}: METADATA=0x{:08X}", SLOT_ID, md);
    // -----End reading key slot metadata-----

    // -----Start push key slot-----
    kmu.keyslot().write(|w| unsafe { w.bits(SLOT_ID) });

    // clear stale events
    kmu.events_pushed().write(|w| w.events_pushed().clear_bit());
    kmu.events_error().write(|w| w.events_error().clear_bit());
    kmu.events_revoked()
        .write(|w| w.events_revoked().clear_bit());

    // trigger push
    kmu.tasks_push().write(|w| w.tasks_push().set_bit());

    // wait for result
    loop {
        if kmu.events_pushed().read().bits() != 0 {
            info!("Slot {} pushed successfully", SLOT_ID);
            kmu.events_pushed().write(|w| w.events_pushed().clear_bit());
            break;
        }
        if kmu.events_error().read().bits() != 0 {
            let es = kmu.events_error().read().bits();
            info!("Push error: 0x{:08X}", es);
            kmu.events_error().write(|w| w.events_error().clear_bit());
            break;
        }
    }

    // read back the 16-byte destination buffer
    unsafe {
        info!("DEST_BUF: {:02X}", DEST_BUF.value);
    }

    // -----End push key slot-----

    loop {
        cortex_m::asm::nop();
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct SrcData {
    /// [0..16): Asset contents or key value.
    /// Used during provisioning and later by the PUSH task.
    pub value: [u8; 16],

    /// [16..20): Revocation policy.
    /// Only the two least-significant bits are used:
    /// 00 = RESERVED, 01 = ROTATING, 10 = LOCKED, 11 = REVOKED
    pub rpolicy: u32,

    /// [20..24): Destination address (must be 128-bit aligned, cannot point to SICR)
    pub dest: u32,

    /// [24..28): Metadata (arbitrary 32 bits, can be read back later)
    pub metadata: u32,
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct DestData {
    pub value: [u8; 16],
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
