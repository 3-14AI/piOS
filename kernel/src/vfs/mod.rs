#![allow(unused_imports)]
#![allow(clippy::result_unit_err)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {

pub type Inode = u64;

pub ghost struct Vfs {
    pub parent_map: Map<Inode, Inode>,
    pub depth_map: Map<Inode, u64>,
    pub locks: Map<Inode, bool>,
}

impl Vfs {
    pub open spec fn valid(&self) -> bool {
        self.parent_map.dom() =~= self.depth_map.dom() &&
        self.parent_map.dom() =~= self.locks.dom() &&
        (forall|ino: Inode| #![auto] self.parent_map.dom().contains(ino) && self.parent_map[ino] != ino ==>
            self.parent_map.dom().contains(self.parent_map[ino]) &&
            self.depth_map[ino] > self.depth_map[self.parent_map[ino]])
    }

    pub proof fn new(root: Inode) -> (v: Self)
        ensures
            v.valid(),
            v.parent_map.dom().contains(root),
            v.parent_map[root] == root,
            v.depth_map[root] == 0,
            v.locks[root] == false,
    {
        let parent_map = Map::empty().insert(root, root);
        let depth_map = Map::empty().insert(root, 0);
        let locks = Map::empty().insert(root, false);

        let vfs = Vfs { parent_map, depth_map, locks };
        vfs
    }

    pub proof fn mkdir(&mut self, parent: Inode, child: Inode) -> (res: Result<(), ()>)
        requires
            old(self).valid(),
        ensures
            self.valid(),
            match res {
                Ok(_) => {
                    old(self).parent_map.dom().contains(parent) &&
                    !old(self).parent_map.dom().contains(child) &&
                    old(self).locks[parent] == true &&
                    self.parent_map.dom() == old(self).parent_map.dom().insert(child) &&
                    self.parent_map[child] == parent &&
                    self.depth_map[child] == old(self).depth_map[parent] + 1 &&
                    self.locks[child] == false
                },
                Err(_) => {
                    *self == *old(self)
                }
            }
    {
        if self.parent_map.dom().contains(parent) && !self.parent_map.dom().contains(child) {
            let parent_locked = self.locks[parent];
            if parent_locked {
                let parent_depth = self.depth_map[parent];
                if parent_depth < 0xffff_ffff_ffff_ffff {
                    let new_depth = (parent_depth + 1) as u64;

                    let mut new_parent_map = self.parent_map;
                    let mut new_depth_map = self.depth_map;
                    let mut new_locks = self.locks;

                    new_parent_map = new_parent_map.insert(child, parent);
                    new_depth_map = new_depth_map.insert(child, new_depth);
                    new_locks = new_locks.insert(child, false);

                    let new_vfs = Vfs {
                        parent_map: new_parent_map,
                        depth_map: new_depth_map,
                        locks: new_locks,
                    };

                    assert forall|ino: Inode| #![auto] new_vfs.parent_map.dom().contains(ino) && new_vfs.parent_map[ino] != ino implies
                        new_vfs.parent_map.dom().contains(new_vfs.parent_map[ino]) &&
                        new_vfs.depth_map[ino] > new_vfs.depth_map[new_vfs.parent_map[ino]] by {
                        if ino == child {
                            assert(new_vfs.parent_map[ino] == parent);
                            assert(new_vfs.depth_map[ino] == new_depth);
                            assert(new_vfs.depth_map[parent] == parent_depth);
                        } else {
                            assert(old(self).parent_map[ino] == new_vfs.parent_map[ino]);
                            let ino_parent = new_vfs.parent_map[ino];
                            if ino_parent == child {
                                // But child is new, so no existing node has it as parent.
                                assert(!old(self).parent_map.dom().contains(child));
                                assert(old(self).parent_map[ino] != child);
                            }
                        }
                    };
                    *self = new_vfs;
                    return Ok(());
                }
            }

        }
        Err(())
    }

    pub proof fn rmdir(&mut self, child: Inode) -> (res: Result<(), ()>)
        requires
            old(self).valid(),
        ensures
            self.valid(),
            match res {
                Ok(_) => {
                    old(self).parent_map.dom().contains(child) &&
                    old(self).parent_map[child] != child && // cannot remove root in this simplified model
                    old(self).locks[child] == true &&
                    self.parent_map.dom() == old(self).parent_map.dom().remove(child) &&
                    self.depth_map.dom() == old(self).depth_map.dom().remove(child) &&
                    self.locks.dom() == old(self).locks.dom().remove(child)
                },
                Err(_) => {
                    *self == *old(self)
                }
            }
    {
        if self.parent_map.dom().contains(child) {
            let is_root = self.parent_map[child] == child;
            if !is_root {
                let child_locked = self.locks[child];
                if child_locked {
                    if forall|ino: Inode| #![auto] self.parent_map.dom().contains(ino) ==> self.parent_map[ino] != child {
                        let mut new_parent_map = self.parent_map;
                        let mut new_depth_map = self.depth_map;
                        let mut new_locks = self.locks;

                        new_parent_map = new_parent_map.remove(child);
                        new_depth_map = new_depth_map.remove(child);
                        new_locks = new_locks.remove(child);

                        let new_vfs = Vfs {
                            parent_map: new_parent_map,
                            depth_map: new_depth_map,
                            locks: new_locks,
                        };

                        assert forall|ino: Inode| #![auto] new_vfs.parent_map.dom().contains(ino) && new_vfs.parent_map[ino] != ino implies
                            new_vfs.parent_map.dom().contains(new_vfs.parent_map[ino]) &&
                            new_vfs.depth_map[ino] > new_vfs.depth_map[new_vfs.parent_map[ino]] by {
                            assert(old(self).parent_map[ino] == new_vfs.parent_map[ino]);
                            assert(old(self).parent_map[new_vfs.parent_map[ino]] == new_vfs.parent_map[new_vfs.parent_map[ino]]);
                        };

                        *self = new_vfs;
                        return Ok(());
                    }
                }
            }
        }
        Err(())
    }

    pub proof fn lock(&mut self, ino: Inode) -> (res: Result<(), ()>)
        requires
            old(self).valid(),
        ensures
            self.valid(),
            match res {
                Ok(_) => {
                    old(self).parent_map.dom().contains(ino) &&
                    old(self).locks[ino] == false &&
                    self.locks[ino] == true &&
                    self.parent_map == old(self).parent_map &&
                    self.depth_map == old(self).depth_map &&
                    self.locks.dom() == old(self).locks.dom()
                },
                Err(_) => {
                    *self == *old(self)
                }
            }
    {
        if self.parent_map.dom().contains(ino) {
            let is_locked = self.locks[ino];
            if !is_locked {
                let mut new_locks = self.locks;
                new_locks = new_locks.insert(ino, true);

                let new_vfs = Vfs {
                    parent_map: self.parent_map,
                    depth_map: self.depth_map,
                    locks: new_locks,
                };

                assert(new_vfs.parent_map.dom() =~= new_vfs.depth_map.dom());
                assert(new_vfs.parent_map.dom() =~= new_vfs.locks.dom());

                assert(new_vfs.parent_map.dom() =~= new_vfs.depth_map.dom());
                assert(new_vfs.parent_map.dom() =~= new_vfs.locks.dom());

                assert forall|ino2: Inode| #![auto] new_vfs.parent_map.dom().contains(ino2) && new_vfs.parent_map[ino2] != ino2 implies
                    new_vfs.parent_map.dom().contains(new_vfs.parent_map[ino2]) &&
                    new_vfs.depth_map[ino2] > new_vfs.depth_map[new_vfs.parent_map[ino2]] by {
                    assert(old(self).parent_map[ino2] == new_vfs.parent_map[ino2]);
                };

                *self = new_vfs;
                return Ok(());
            }
        }
        Err(())
    }

    pub proof fn unlock(&mut self, ino: Inode) -> (res: Result<(), ()>)
        requires
            old(self).valid(),
        ensures
            self.valid(),
            match res {
                Ok(_) => {
                    old(self).parent_map.dom().contains(ino) &&
                    old(self).locks[ino] == true &&
                    self.locks[ino] == false &&
                    self.parent_map == old(self).parent_map &&
                    self.depth_map == old(self).depth_map &&
                    self.locks.dom() == old(self).locks.dom()
                },
                Err(_) => {
                    *self == *old(self)
                }
            }
    {
        if self.parent_map.dom().contains(ino) {
            let is_locked = self.locks[ino];
            if is_locked {
                let mut new_locks = self.locks;
                new_locks = new_locks.insert(ino, false);

                let new_vfs = Vfs {
                    parent_map: self.parent_map,
                    depth_map: self.depth_map,
                    locks: new_locks,
                };

                assert forall|ino2: Inode| #![auto] new_vfs.parent_map.dom().contains(ino2) && new_vfs.parent_map[ino2] != ino2 implies
                    new_vfs.parent_map.dom().contains(new_vfs.parent_map[ino2]) &&
                    new_vfs.depth_map[ino2] > new_vfs.depth_map[new_vfs.parent_map[ino2]] by {
                    assert(old(self).parent_map[ino2] == new_vfs.parent_map[ino2]);
                };

                *self = new_vfs;
                return Ok(());
            }
        }
        Err(())
    }
}
} // verus!

#[cfg(not(feature = "verus"))]
extern crate alloc;

#[cfg(not(feature = "verus"))]
pub struct Vfs {
    pub parent_map: alloc::collections::BTreeMap<u64, u64>,
    pub depth_map: alloc::collections::BTreeMap<u64, u64>,
    pub locks: alloc::collections::BTreeMap<u64, bool>,
}

#[cfg(not(feature = "verus"))]
impl Vfs {
    pub fn new(root: u64) -> Self {
        let mut parent_map = alloc::collections::BTreeMap::new();
        let mut depth_map = alloc::collections::BTreeMap::new();
        let mut locks = alloc::collections::BTreeMap::new();

        parent_map.insert(root, root);
        depth_map.insert(root, 0);
        locks.insert(root, false);

        Vfs {
            parent_map,
            depth_map,
            locks,
        }
    }

    pub fn mkdir(&mut self, parent: u64, child: u64) -> Result<(), ()> {
        if self.parent_map.contains_key(&parent) && !self.parent_map.contains_key(&child) {
            let parent_locked = *self.locks.get(&parent).unwrap();
            if parent_locked {
                let parent_depth = *self.depth_map.get(&parent).unwrap();
                if parent_depth < 0xffff_ffff_ffff_ffff {
                    self.parent_map.insert(child, parent);
                    self.depth_map.insert(child, parent_depth + 1);
                    self.locks.insert(child, false);
                    return Ok(());
                }
            }
        }
        Err(())
    }

    pub fn rmdir(&mut self, child: u64) -> Result<(), ()> {
        if self.parent_map.contains_key(&child) {
            let parent = *self.parent_map.get(&child).unwrap();
            if parent != child {
                let child_locked = *self.locks.get(&child).unwrap();
                if child_locked {
                    // Check leaf
                    let mut is_leaf = true;
                    for (_, &p) in self.parent_map.iter() {
                        if p == child {
                            is_leaf = false;
                            break;
                        }
                    }
                    if is_leaf {
                        self.parent_map.remove(&child);
                        self.depth_map.remove(&child);
                        self.locks.remove(&child);
                        return Ok(());
                    }
                }
            }
        }
        Err(())
    }

    pub fn lock(&mut self, ino: u64) -> Result<(), ()> {
        if self.parent_map.contains_key(&ino) {
            let is_locked = *self.locks.get(&ino).unwrap();
            if !is_locked {
                self.locks.insert(ino, true);
                return Ok(());
            }
        }
        Err(())
    }

    pub fn unlock(&mut self, ino: u64) -> Result<(), ()> {
        if self.parent_map.contains_key(&ino) {
            let is_locked = *self.locks.get(&ino).unwrap();
            if is_locked {
                self.locks.insert(ino, false);
                return Ok(());
            }
        }
        Err(())
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_basic() {
        let mut vfs = Vfs::new(1);

        // Cannot mkdir without lock
        assert!(vfs.mkdir(1, 2).is_err());

        // Lock root
        assert!(vfs.lock(1).is_ok());

        // Create child
        assert!(vfs.mkdir(1, 2).is_ok());

        // Cannot lock twice
        assert!(vfs.lock(1).is_err());

        // Cannot create if child already exists
        assert!(vfs.mkdir(1, 2).is_err());

        // Root cannot be removed
        assert!(vfs.rmdir(1).is_err());

        // Cannot remove child if not locked
        assert!(vfs.rmdir(2).is_err());

        // Lock child
        assert!(vfs.lock(2).is_ok());

        // Remove child
        assert!(vfs.rmdir(2).is_ok());

        // Unlock root
        assert!(vfs.unlock(1).is_ok());

        // Cannot unlock twice
        assert!(vfs.unlock(1).is_err());
    }
}

#[cfg(not(feature = "verus"))]
pub mod semantic;
