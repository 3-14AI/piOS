#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct Ext4 {
        pub mounted: bool,
    }

    impl Ext4 {
        pub fn new() -> (d: Self)
            ensures d.mounted == false
        {
            Ext4 { mounted: false }
        }

        pub fn mount(&mut self) -> (res: Result<(), ()>)
            ensures
                match res {
                    Ok(_) => self.mounted == true,
                    Err(_) => *self == *old(self),
                }
        {
            let mut new_ext4 = Ext4 { mounted: true };
            *self = new_ext4;
            Ok(())
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct Ext4 {}

#[cfg(not(feature = "verus"))]
impl Ext4 {
    pub fn new() -> Self {
        Ext4 {}
    }
    pub fn mount(&self) -> Result<(), ()> {
        Ok(())
    }
    pub fn fsck(&self) -> Result<(), ()> {
        Ok(())
    }
    pub fn journal(&self) -> Result<(), ()> {
        Ok(())
    }
}

#[cfg(not(feature = "verus"))]
impl Default for Ext4 {
    fn default() -> Self {
        Self::new()
    }
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
