#![no_std]

pub fn common_init() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_init() {
        assert!(common_init());
    }
}
