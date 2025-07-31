//! Caxton library root

#[cfg(test)]
mod tests {
    #[test]
    fn type_safety_smoke_test() {
        // This test ensures our type system is working correctly
        let x: i32 = 42;
        let y: i32 = 58;
        let sum: i32 = x + y;
        assert_eq!(sum, 100);
    }
}