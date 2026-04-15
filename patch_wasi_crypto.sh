sed -i '/assert_eq!(constant_time_eq(&a, &c), 0);/a\
        #[cfg(not(feature = "verus"))]\
        assert_eq!(constant_time_eq(&a, &d), 0);' kernel/src/wasm/wasi_crypto.rs
