#![no_std]
#![no_main]

// use core::ptr;
use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use nrf54l15_app_pac;
use panic_probe as _;

const OPEADDR_SCALAR_MUL_P256: u32 = 0x00;

/// High level P-256 interface on top of CRACEN PK.
pub struct P256<'a> {
    /// Top level CRACEN control block at 0x5004_8000
    cracen: &'a nrf54l15_app_pac::GlobalCracenS,
    /// Crypto core at 0x5180_0000
    core: &'a nrf54l15_app_pac::GlobalCracencoreS,
}

impl<'a> P256<'a> {
    /// Create handle. Caller guarantees unique mutable access to CRACEN and CRACENCORE.
    pub fn new(
        cracen: &'a nrf54l15_app_pac::GlobalCracenS,
        core: &'a nrf54l15_app_pac::GlobalCracencoreS,
    ) -> Self {
        Self { cracen, core }
    }

    /// Enable the hardware blocks we need.
    pub fn enable(&self) {
        // SAFETY: we have unique access via &mut in normal code, but PAC uses
        // interior mutability (read/write methods). svd2rust handles that.
        self.cracen.enable().write(|w| {
            w.cryptomaster().enabled();
            w.rng().disabled();
            w.pkeikg().enabled()
        });
    }

    /// Load arbitrary data from system RAM into a PK operand slot in CRACEN internal RAM.
    pub fn dma_fetch_into_pk(&self, src: *const u8, byte_len: usize, pk_slot_addr: u32) {
        let dma = self.core.cryptmstrdma();

        let src_addr = src as u32;
        // Source: system RAM
        dma.fetchaddrlsb()
            .write(|w| unsafe { w.fetchaddrlsb().bits(src_addr) });
        // dma.fetchaddrmsb()
        //     .write(|w| unsafe { w.fetchaddrmsb().bits(0) });

        // Destination: PK RAM slot
        // dma.fetchpkaddrlsb()
        //     .write(|w| unsafe { w.fetchpkaddrlsb().bits(pk_slot_addr) });
        // dma.fetchpkaddrmsb()
        //     .write(|w| unsafe { w.fetchpkaddrmsb().bits(0) });

        // Program length and flags.
        dma.fetchlen().write(|w| {
            // FETCHLEN[27:0] number of bytes.
            unsafe { w.fetchlen().bits(byte_len as u32) }
                // 0 = use address from FETCHADDRLSB
                .fetchcstaddr()
                .clear_bit()
                // 0 = no realign
                .fetchrealign()
                .clear_bit()
                // 0 = zero padding disabled
                .fetchzpadding()
                .clear_bit()
        });

        dma.start().write(|w| w.startfetch().set_bit());

        while dma.status().read().fetchbusy().bit_is_set() {}
    }

    /// Read data out of a PK operand slot back into system RAM.
    pub fn dma_push_from_pk(&self, dst: *mut u8, byte_len: usize, pk_slot_addr: u32) {
        let dma = self.core.cryptmstrdma();

        let dst_addr = dst as u32;
        // Destination: system RAM
        dma.pushaddrlsb()
            .write(|w| unsafe { w.pushaddrlsb().bits(dst_addr) });
        // dma.pushaddrmsb()
        //     .write(|w| unsafe { w.pushaddrmsb().bits(0) });

        // // Source: PK RAM slot
        // dma.pushpkaddrlsb()
        //     .write(|w| unsafe { w.pushpkaddrlsb().bits(pk_slot_addr) });
        // dma.pushpkaddrmsb()
        //     .write(|w| unsafe { w.pushpkaddrmsb().bits(0) });

        dma.pushlen().write(|w| {
            unsafe { w.pushlen().bits(byte_len as u32) }
                .pushcstaddr()
                .clear_bit()
                .pushrealign()
                .clear_bit()
                // PUSHDISCARD=0 means data is actually written to dst
                .pushdiscard()
                .clear_bit()
        });

        dma.start().write(|w| w.startpush().set_bit());

        while dma.status().read().pushbusy().bit_is_set() {}
    }

    /// Configure PK pointer slots and operand size for a P-256 scalar mult.
    ///
    /// For ECDH you want Q = d * P where:
    /// - d is our private scalar (32 bytes)
    /// - P is peer public point (x,y 32 byte each)
    /// - Q.x is the shared secret X coordinate
    ///
    /// The PK.POINTERS register encodes which internal slot holds each operand:
    /// - OPPTRA  bits [3:0]  -> operand A
    /// - OPPTRB  bits [7:4]  -> operand B
    /// - OPPTRC  bits [19:16] -> result
    /// - OPPTRN  bits [27:24] -> modulus
    ///
    /// The SVD text states:
    ///  OPPTRA: "operand A is located in memory (location 0x0 to 0xF)"
    ///  OPPTRB: "operand B is located in memory (location 0x0 to 0xF)"
    ///  OPPTRC: "result will be stored in memory (0x0 to 0xF)"
    ///  OPPTRN: "modulus is located in memory (0x0 to 0xF)"
    ///
    /// You must supply these 4 slot indices. The mapping of big integers and point coordinates
    /// to slots is operation dependent and not documented in the SVD.
    pub fn setup_pk_pointers(&self, slot_a: u8, slot_b: u8, slot_c: u8, slot_n: u8) {
        let pk = self.core.pk();

        pk.pointers().write(|w| {
            unsafe {
                w.opptra().bits(slot_a & 0xF);
                w.opptrb().bits(slot_b & 0xF);
                w.opptrc().bits(slot_c & 0xF);
                w.opptrn().bits(slot_n & 0xF);
            }
            w
        });
    }

    /// Program the PK.COMMAND register for a P-256 operation.
    pub fn setup_pk_command_p256(
        &self,
        opcode: u8,
        operand_size_bytes: u16, // 32 for P-256 scalars and coordinates
    ) {
        let pk = self.core.pk();

        let opbytesm1 = operand_size_bytes - 1;

        pk.command().write(|w| {
            unsafe {
                // 7-bit OPEADDR
                w.opeaddr().bits(opcode);

                // prime field GF(p)
                w.fieldf().clear_bit();

                // OPBYTESM1[15:8]
                w.opbytesm1().bits(opbytesm1);

                // RANDMOD off
                w.randmod().clear_bit();

                // SELCURVE = P256 (0x1)
                w.selcurve().bits(0x1);

                // RANDKE off
                w.randke().clear_bit();

                // RANDPROJ off
                w.randproj().clear_bit();

                // EDWARDS off for Weierstrass P-256
                w.edwards().clear_bit();

                // SWAPBYTES = native little endian
                w.swapbytes().native();

                // FLAGA / FLAGB cleared
                w.flaga().clear_bit();
                w.flagb().clear_bit();

                // CALCR2 = NRECALCULATE (0)
                w.calcr2().nrecalculate();
            }
            w
        });
    }

    /// Set PK.OPSIZE to match operand byte length.
    pub fn set_opsize(&self) {
        let pk = self.core.pk();
        pk.opsize().write(|w| unsafe {
            w.opsize()
                .bits(nrf54l15_app_pac::global_cracencore_s::pk::opsize::Opsize::Opsize256.into())
        });
    }

    /// Kick the PK core and block until done.
    pub fn run_and_wait(&self) -> Result<(), PkError> {
        let pk = self.core.pk();

        // Clear any stale IRQ flag
        pk.control().write(|w| w.clearirq().set_bit());

        // START = 1
        pk.control().write(|w| w.start().set_bit());

        // wait PKBUSY == 0
        while pk.status().read().pkbusy().bit_is_set() {}

        // Check ERRORFLAGS
        let err = pk.status().read().errorflags().bits();
        if err != 0 {
            let failptr = pk.status().read().failptr().bits() as u8;
            return Err(PkError {
                errorflags: err,
                fail_slot: failptr,
            });
        }

        Ok(())
    }

    /// High level ECDH primitive skeleton.
    pub fn ecdh_p256(
        &self,
        private_key: &[u8; 32],
        peer_pub_x: &[u8; 32],
        peer_pub_y: &[u8; 32],
        out_shared_x: &mut [u8; 32],
    ) -> Result<(), PkError> {
        // 1. load operands into PK RAM slots
        self.dma_fetch_into_pk(private_key.as_ptr(), 32, 0);
        self.dma_fetch_into_pk(peer_pub_x.as_ptr(), 32, 1);
        self.dma_fetch_into_pk(peer_pub_y.as_ptr(), 32, 3);

        // 2. program PK pointers
        self.setup_pk_pointers(
            0, // OPPTRA = scalar d
            1, // OPPTRB = input point
            2, // OPPTRC = result point
            3, // OPPTRN = modulus p
        );

        // 3. operand size
        self.set_opsize();

        // 4. command. opcode for "scalar mult" is not in SVD,
        //    so we pass a placeholder 0x00.
        self.setup_pk_command_p256(0x00, 32);

        // 5. run
        self.run_and_wait()?;

        // 6. read result back
        self.dma_push_from_pk(out_shared_x.as_mut_ptr(), 32, 2);

        Ok(())
    }
}

/// Basic PK error info after run_and_wait.
#[derive(Debug)]
pub struct PkError {
    pub errorflags: u16,
    pub fail_slot: u8,
}

#[entry]
fn main() -> ! {
    // take peripherals
    let peripherals = nrf54l15_app_pac::Peripherals::take().unwrap();

    // build handle
    let p256 = P256::new(
        &peripherals.global_cracen_s,
        &peripherals.global_cracencore_s,
    );

    // enable crypto hardware
    p256.enable();

    // dummy keys
    let my_priv = [0u8; 32];
    let peer_x = [0u8; 32];
    let peer_y = [0u8; 32];
    let mut shared_x = [0u8; 32];

    // run scalar multiply
    match p256.ecdh_p256(&my_priv, &peer_x, &peer_y, &mut shared_x) {
        Ok(()) => info!("ECDH result: {:02x}", shared_x),
        Err(e) => info!(
            "ECDH failed -- e.errorflags: {:b}, e.fail_slot: {:b}",
            e.errorflags, e.fail_slot
        ),
    }

    loop {}
}
