#![no_std]
#![no_main]

use defmt::info;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::pac::vpr::vals;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Configure VPR core to secure mode");
    let spu = embassy_nrf::pac::SPU00_S;

    let flpr_index = 12;

    spu.periph(flpr_index).perm().write(|w| {
        w.set_secattr(true);
        w.set_dmasec(true);
    });

    let vpr = embassy_nrf::pac::VPR00_S;

    // Start the riscv core
    const RISCV_ENTRY_ADDR: u32 = 0x00010000;

    info!("Start VPR core from address {:#010x}", RISCV_ENTRY_ADDR);

    vpr.initpc().write(|w| {
        *w = RISCV_ENTRY_ADDR;
    });

    vpr.cpurun().write(|w| w.set_en(vals::CpurunEn::RUNNING));
}
