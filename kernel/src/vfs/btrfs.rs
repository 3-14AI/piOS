#[cfg(not(feature = "verus"))]
pub struct Btrfs {}

#[cfg(not(feature = "verus"))]
impl Btrfs {
    pub fn new() -> Self { Btrfs {} }
    pub fn mount(&self) -> Result<(), ()> { Ok(()) }
    pub fn fsck(&self) -> Result<(), ()> { Ok(()) }
    pub fn snapshot(&self) -> Result<(), ()> { Ok(()) }
}

#[cfg(not(feature = "verus"))]
impl Default for Btrfs {
    fn default() -> Self { Self::new() }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_btrfs() {
        let fs = Btrfs::new();
        assert!(fs.mount().is_ok());
        assert!(fs.fsck().is_ok());
        assert!(fs.snapshot().is_ok());
    }
}
