#[cfg(not(feature = "verus"))]
pub struct Zfs {}

#[cfg(not(feature = "verus"))]
impl Zfs {
    pub fn new() -> Self {
        Zfs {}
    }
    pub fn mount(&self) -> Result<(), ()> {
        Ok(())
    }
    pub fn fsck(&self) -> Result<(), ()> {
        Ok(())
    }
    pub fn snapshot(&self) -> Result<(), ()> {
        Ok(())
    }
}

#[cfg(not(feature = "verus"))]
impl Default for Zfs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_zfs() {
        let fs = Zfs::new();
        assert!(fs.mount().is_ok());
        assert!(fs.fsck().is_ok());
        assert!(fs.snapshot().is_ok());
    }
}
