#![allow(dead_code)]

#[cfg(not(feature = "verus"))]
use crate::acpi::AcpiTables;
#[cfg(not(feature = "verus"))]
use tracing::{info, debug};

#[cfg(not(feature = "verus"))]
pub fn init_smp(tables: &AcpiTables) {
    info!("Initializing Symmetric Multiprocessing (SMP)...");

    let apic_ids = &tables.processor_apic_ids;
    let count = tables.processor_count;
    info!("Discovered {} local APICs", count);

    for i in 0..count {
        let apic_id = apic_ids[i];
        if apic_id == 0 { // Assume 0 is BSP for now
            continue;
        }
        debug!("Waking up AP with APIC ID {}", apic_id);
        wakeup_ap(apic_id);
    }

    info!("SMP Scheduler initialized with APIC support.");
}

#[cfg(not(feature = "verus"))]
fn wakeup_ap(apic_id: u8) {
    let local_apic_base: usize = 0xFEE00000;
    let icr_low = (local_apic_base + 0x300) as *mut u32;
    let icr_high = (local_apic_base + 0x310) as *mut u32;

    unsafe {
        // High register: Set destination APIC ID
        core::ptr::write_volatile(icr_high, (apic_id as u32) << 24);

        // Low register: INIT, Physical, Assert, Edge, All Excluding Self
        core::ptr::write_volatile(icr_low, 0x00004500);

        // Wait a bit (simulate 10ms delay for INIT)
        for _ in 0..10000 {
            core::hint::spin_loop();
        }

        // Send SIPI
        let vector = 0x08; // Example vector for SIPI (start address 0x8000)
        core::ptr::write_volatile(icr_high, (apic_id as u32) << 24);
        core::ptr::write_volatile(icr_low, 0x00004600 | vector);

        // Wait a bit (simulate 200us delay)
        for _ in 0..200 {
            core::hint::spin_loop();
        }

        // Send second SIPI
        core::ptr::write_volatile(icr_high, (apic_id as u32) << 24);
        core::ptr::write_volatile(icr_low, 0x00004600 | vector);
    }
    debug!("Sent INIT-SIPI-SIPI sequence to APIC ID {}", apic_id);
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::acpi::AcpiTables;

    #[test]
    fn test_smp_init() {
        // Avoid running wakeup_ap in tests since it writes to physical memory addresses
        let mut tables = AcpiTables::parse();
        tables.processor_count = 1;
        tables.processor_apic_ids[0] = 0;
        init_smp(&tables); // Only BSP, so wakeup_ap is not called
    }
}
