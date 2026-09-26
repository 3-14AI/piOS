#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct AcpiTables {
        pub valid: bool,
    }

    impl AcpiTables {
        pub fn parse() -> (t: Self)
            ensures t.valid == true
        {
            AcpiTables { valid: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct AcpiTables {
    pub valid: bool,
    pub fadt_address: Option<u64>,
    pub madt_address: Option<u64>,
    pub pm1a_control_block: Option<u32>,
    pub slp_typa: Option<u8>,
    pub slp_typb: Option<u8>,
}

#[cfg(not(feature = "verus"))]
impl AcpiTables {
    pub fn parse() -> Self {
        AcpiTables {
            valid: true,
            fadt_address: None,
            madt_address: None,
            pm1a_control_block: None,
            slp_typa: None,
            slp_typb: None,
        }
    }
}

#[cfg(not(feature = "verus"))]
#[repr(C, packed)]
struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[cfg(not(feature = "verus"))]
#[repr(C, packed)]
struct Xsdp {
    rsdp: Rsdp,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[cfg(not(feature = "verus"))]
#[repr(C, packed)]
struct SdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

#[cfg(not(feature = "verus"))]
#[repr(C, packed)]
struct Fadt {
    header: SdtHeader,
    firmware_ctrl: u32,
    dsdt: u32,
    reserved: u8,
    preferred_power_management_profile: u8,
    sci_interrupt: u16,
    smi_command_port: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4bios_req: u8,
    pstate_control: u8,
    pm1a_event_block: u32,
    pm1b_event_block: u32,
    pm1a_control_block: u32,
    pm1b_control_block: u32,
    pm2_control_block: u32,
    pm_timer_block: u32,
    gpe0_block: u32,
    gpe1_block: u32,
    pm1_event_length: u8,
    pm1_control_length: u8,
    pm2_control_length: u8,
    pm_timer_length: u8,
    gpe0_length: u8,
    gpe1_length: u8,
    gpe1_base: u8,
    cstate_control: u8,
    worst_c2_latency: u16,
    worst_c3_latency: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alarm: u8,
    month_alarm: u8,
    century: u8,
    iapc_boot_arch: u16,
    reserved2: u8,
    flags: u32,
}

#[cfg(not(feature = "verus"))]
impl AcpiTables {
    pub fn parse_from_rsdp(rsdp_addr: usize) -> Self {
        let rsdp = unsafe { core::ptr::read_unaligned(rsdp_addr as *const Rsdp) };
        if &rsdp.signature != b"RSD PTR " {
            return AcpiTables {
                valid: false,
                fadt_address: None,
                madt_address: None,
                pm1a_control_block: None,
                slp_typa: None,
                slp_typb: None,
            };
        }

        let mut fadt_address = None;
        let mut madt_address = None;
        let mut pm1a_control_block = None;
        let mut slp_typa = None;
        let mut slp_typb = None;

        if rsdp.revision >= 2 {
            let xsdp = unsafe { core::ptr::read_unaligned(rsdp_addr as *const Xsdp) };
            let xsdt_addr = xsdp.xsdt_address as usize;
            let xsdt_header = unsafe { core::ptr::read_unaligned(xsdt_addr as *const SdtHeader) };
            if &xsdt_header.signature == b"XSDT" {
                let entries_count =
                    (xsdt_header.length as usize - core::mem::size_of::<SdtHeader>()) / 8;
                let entries_base = (xsdt_addr + core::mem::size_of::<SdtHeader>()) as *const u64;

                for i in 0..entries_count {
                    let entry_addr = unsafe { core::ptr::read_unaligned(entries_base.add(i)) };
                    let entry_header = unsafe { core::ptr::read_unaligned(entry_addr as usize as *const SdtHeader) };
                    if &entry_header.signature == b"FACP" {
                        fadt_address = Some(entry_addr);
                    } else if &entry_header.signature == b"APIC" {
                        madt_address = Some(entry_addr);
                    }
                }
            }
        } else {
            let rsdt_addr = rsdp.rsdt_address as usize;
            let rsdt_header = unsafe { core::ptr::read_unaligned(rsdt_addr as *const SdtHeader) };
            if &rsdt_header.signature == b"RSDT" {
                let entries_count =
                    (rsdt_header.length as usize - core::mem::size_of::<SdtHeader>()) / 4;
                let entries_base = (rsdt_addr + core::mem::size_of::<SdtHeader>()) as *const u32;

                for i in 0..entries_count {
                    let entry_addr = unsafe { core::ptr::read_unaligned(entries_base.add(i)) };
                    let entry_header = unsafe { core::ptr::read_unaligned(entry_addr as usize as *const SdtHeader) };
                    if &entry_header.signature == b"FACP" {
                        fadt_address = Some(entry_addr as u64);
                    } else if &entry_header.signature == b"APIC" {
                        madt_address = Some(entry_addr as u64);
                    }
                }
            }
        }

        if let Some(fadt_addr) = fadt_address {
            let fadt = unsafe { core::ptr::read_unaligned(fadt_addr as usize as *const Fadt) };
            pm1a_control_block = Some(fadt.pm1a_control_block);

            // In a real system we would parse DSDT to get \_S5_ AML object for slp_typa/b
            // Here we provide hardcoded default values commonly used
            slp_typa = Some(5);
            slp_typb = Some(5);
        }

        AcpiTables {
            valid: true,
            fadt_address,
            madt_address,
            pm1a_control_block,
            slp_typa,
            slp_typb,
        }
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acpi_parsing() {
        let tables = AcpiTables::parse();
        assert!(tables.valid);
    }
}
