extern crate alloc;
use alloc::vec::Vec;
use vstd::prelude::*;

verus! {

pub struct WasmMemory {
    pub memory: Vec<u8>,
}

impl WasmMemory {
    pub fn new() -> (res: Self)
        ensures res.memory@.len() == 0,
    {
        WasmMemory {
            memory: Vec::new(),
        }
    }

    pub fn len(&self) -> (res: usize)
        ensures res == self.memory@.len(),
    {
        self.memory.len()
    }

    pub fn read(&self, offset: usize, length: usize) -> (res: Option<Vec<u8>>)
        ensures
            offset + length <= self.memory@.len() ==> (matches!(res, Some(r)) && res.unwrap()@.len() == length &&
                (forall|k: int| 0 <= k < length ==> res.unwrap()@[k] == self.memory@[offset + k])),
            offset + length > self.memory@.len() ==> matches!(res, None),
    {
        let mem_len = self.memory.len();
        if offset <= mem_len && length <= mem_len - offset {
            let mut result = Vec::new();
            let mut i: usize = 0;
            while i < length
                invariant
                    offset <= mem_len,
                    length <= mem_len - offset,
                    0 <= i <= length,
                    mem_len == self.memory@.len(),
                    result@.len() == i,
                    forall|k: int| 0 <= k < i ==> result@[k] == self.memory@[offset + k],
                decreases length - i,
            {
                let idx: usize = offset + i;
                result.push(self.memory[idx]);
                i += 1;
            }
            Some(result)
        } else {
            None
        }
    }

    pub fn write(&mut self, offset: usize, data: &Vec<u8>) -> (res: Result<(), ()>)
        ensures
            offset + data@.len() <= old(self).memory@.len() ==> (matches!(res, Ok(())) && self.memory@.len() == old(self).memory@.len() &&
                (forall|k: int| 0 <= k < data@.len() ==> self.memory@[offset + k] == data@[k]) &&
                (forall|k: int| 0 <= k < old(self).memory@.len() && (k < offset || k >= offset + data@.len()) ==> self.memory@[k] == old(self).memory@[k])),
            offset + data@.len() > old(self).memory@.len() ==> (matches!(res, Err(())) && self.memory@ =~= old(self).memory@),
    {
        let mem_len = self.memory.len();
        let data_len = data.len();
        if offset <= mem_len && data_len <= mem_len - offset {
            let mut i: usize = 0;
            while i < data_len
                invariant
                    offset <= mem_len,
                    data_len <= mem_len - offset,
                    0 <= i <= data_len,
                    mem_len == old(self).memory@.len(),
                    data_len == data@.len(),
                    self.memory@.len() == old(self).memory@.len(),
                    forall|k: int| 0 <= k < i ==> self.memory@[offset + k] == data@[k],
                    forall|k: int| 0 <= k < old(self).memory@.len() && (k < offset || k >= offset + i) ==> self.memory@[k] == old(self).memory@[k],
                decreases data_len - i,
            {
                let idx: usize = offset + i;
                self.memory.set(idx, data[i]);
                i += 1;
            }
            Ok(())
        } else {
            Err(())
        }
    }
}

} // verus!
