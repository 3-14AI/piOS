#[cfg(not(feature = "verus"))]
pub struct Ext4 {}

#[cfg(not(feature = "verus"))]
impl Ext4 {
    pub fn new() -> Self { Ext4 {} }
    pub fn mount(&self) -> Result<(), ()> { Ok(()) }
    pub fn fsck(&self) -> Result<(), ()> { Ok(()) }
    pub fn journal(&self) -> Result<(), ()> { Ok(()) }
}

#[cfg(not(feature = "verus"))]
impl Default for Ext4 {
    fn default() -> Self { Self::new() }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ext4() {
        let fs = Ext4::new();
        assert!(fs.mount().is_ok());
        assert!(fs.fsck().is_ok());
        assert!(fs.journal().is_ok());
    }
}
