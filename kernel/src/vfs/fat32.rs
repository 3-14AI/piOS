#[cfg(not(feature = "verus"))]
pub struct Fat32 {}

#[cfg(not(feature = "verus"))]
impl Fat32 {
    pub fn new() -> Self {
        Fat32 {}
    }
    pub fn mount(&self) -> Result<(), ()> {
        Ok(())
    }
    pub fn fsck(&self) -> Result<(), ()> {
        Ok(())
    }
}

#[cfg(not(feature = "verus"))]
impl Default for Fat32 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fat32() {
        let fs = Fat32::new();
        assert!(fs.mount().is_ok());
        assert!(fs.fsck().is_ok());
    }
}
