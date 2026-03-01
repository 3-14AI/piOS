fn main() {
    println!("piOS builder tool");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_builder_basic() {
        super::main();
        assert_eq!(1 + 1, 2);
    }
}
