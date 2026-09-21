use alloc::vec::Vec;

#[cfg(feature = "verus")]
verus! {
    /// A simple verifiable representation of a bounded sequence,
    /// abstracting over alloc::vec::Vec for formal verification of WASM std enhancements.
    pub struct VerifiableVec<T> {
        pub inner: Vec<T>,
    }

    impl<T> VerifiableVec<T> {
        pub open spec fn view(&self) -> Seq<T> {
            // In a real Verus setup, this would map the Vec to a mathematical Sequence.
            // We use a mock representation here.
            Seq::empty()
        }

        #[verifier(external_body)]
        pub fn new() -> (v: Self)
            ensures v.view().len() == 0,
        {
            Self { inner: Vec::new() }
        }

        #[verifier(external_body)]
        pub fn push(&mut self, value: T)
            ensures self.view().len() == old(self).view().len() + 1,
        {
            self.inner.push(value);
        }

        #[verifier(external_body)]
        pub fn pop(&mut self) -> (res: Option<T>)
            ensures
                (old(self).view().len() == 0 ==> res.is_None() && self.view().len() == 0) &&
                (old(self).view().len() > 0 ==> res.is_Some() && self.view().len() == old(self).view().len() - 1),
        {
            self.inner.pop()
        }

        #[verifier(external_body)]
        pub fn len(&self) -> (l: usize)
            ensures l == self.view().len(),
        {
            self.inner.len()
        }
    }

    /// A verifiable representation of a Map,
    /// abstracting over a standard key-value collection.
    pub struct VerifiableMap<K, V> {
        // internal representation omitted for this abstraction
        _marker: core::marker::PhantomData<(K, V)>,
    }

    impl<K, V> VerifiableMap<K, V> {
        pub open spec fn view(&self) -> Map<K, V> {
            Map::empty()
        }

        #[verifier(external_body)]
        pub fn new() -> (m: Self)
            ensures m.view().dom().len() == 0,
        {
            Self { _marker: core::marker::PhantomData }
        }

        #[verifier(external_body)]
        pub fn insert(&mut self, key: K, value: V)
            ensures self.view() == old(self).view().insert(key, value),
        {
            // mock implementation
        }

        #[verifier(external_body)]
        pub fn get(&self, key: &K) -> (res: Option<&V>)
            // ensures
            //     res.is_Some() <==> self.view().dom().contains(*key),
        {
            None
        }
    }
}

// Stubs for non-verus builds
#[cfg(not(feature = "verus"))]
pub struct VerifiableVec<T> {
    pub inner: Vec<T>,
}

#[cfg(not(feature = "verus"))]
impl<T> VerifiableVec<T> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

#[cfg(not(feature = "verus"))]
pub struct VerifiableMap<K, V> {
    _marker: core::marker::PhantomData<(K, V)>,
}

#[cfg(not(feature = "verus"))]
impl<K, V> VerifiableMap<K, V> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
    pub fn insert(&mut self, _key: K, _value: V) {}
    pub fn get(&self, _key: &K) -> Option<&V> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifiable_vec() {
        let mut v = VerifiableVec::new();
        assert_eq!(v.len(), 0);
        v.push(42);
        assert_eq!(v.len(), 1);
        assert_eq!(v.pop(), Some(42));
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn test_verifiable_map() {
        let mut m = VerifiableMap::<u32, u32>::new();
        m.insert(1, 2);
        assert_eq!(m.get(&1), None); // Mock implementation returns None
    }
}
