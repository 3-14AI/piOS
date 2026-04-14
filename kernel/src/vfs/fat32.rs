#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct Fat32 {
        pub mounted: bool,
    }

    impl Fat32 {
        pub fn new() -> (d: Self)
            ensures d.mounted == false
        {
            Fat32 { mounted: false }
        }

        pub fn mount(&mut self) -> (res: Result<(), ()>)
            ensures
                match res {
                    Ok(_) => self.mounted == true,
                    Err(_) => *self == *old(self),
                }
        {
            let mut new_fat = Fat32 { mounted: true };
            *self = new_fat;
            Ok(())
        }
    }
}

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
