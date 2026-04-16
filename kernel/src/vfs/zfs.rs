#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct Zfs {
        pub mounted: bool,
    }

    impl Zfs {
        pub fn new() -> (d: Self)
            ensures d.mounted == false
        {
            Zfs { mounted: false }
        }

        pub fn mount(&mut self) -> (res: Result<(), ()>)
            ensures
                match res {
                    Ok(_) => self.mounted == true,
                    Err(_) => *self == *old(self),
                }
        {
            let mut new_zfs = Zfs { mounted: true };
            *self = new_zfs;
            Ok(())
        }

        pub fn fsck(&self) -> (res: Result<(), ()>) { Ok(()) }

        pub fn snapshot(&self) -> (res: Result<(), ()>) { Ok(()) }
    }
}

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
