#![no_std]

extern crate alloc;

pub fn api_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version() {
        assert_eq!(api_version(), 1);
    }
}

pub mod a2a;
pub mod mcp;
