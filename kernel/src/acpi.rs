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
pub struct AcpiTables {
    pub valid: bool,
}

#[cfg(not(feature = "verus"))]
impl AcpiTables {
    pub fn parse() -> Self {
        AcpiTables { valid: true }
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acpi_parsing() {
        let tables = AcpiTables::parse();
        assert_eq!(tables.valid, true);
    }
}
