#![cfg_attr(target_arch = "wasm32", no_std)]

pub fn init_ide() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ide_init() {
        assert!(init_ide());
    }
}
