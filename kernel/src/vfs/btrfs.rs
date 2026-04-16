#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct Btrfs {
        pub mounted: bool,
    }

    impl Btrfs {
        pub fn new() -> (d: Self)
            ensures d.mounted == false
        {
            Btrfs { mounted: false }
        }

        pub fn mount(&mut self) -> (res: Result<(), ()>)
            ensures
                match res {
                    Ok(_) => self.mounted == true,
                    Err(_) => *self == *old(self),
                }
        {
            let mut new_btrfs = Btrfs { mounted: true };
            *self = new_btrfs;
            Ok(())
        }

        pub fn fsck(&self) -> (res: Result<(), ()>) { Ok(()) }

        pub fn snapshot(&self) -> (res: Result<(), ()>) { Ok(()) }
    }
}

#[cfg(not(feature = "verus"))]
pub struct Btrfs {}

#[cfg(not(feature = "verus"))]
impl Btrfs {
    pub fn new() -> Self {
        Btrfs {}
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
impl Default for Btrfs {
    fn default() -> Self {
        Self::new()
    }
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
