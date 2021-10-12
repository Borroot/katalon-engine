pub fn addition(a: u8, b: u8) -> u8 {
    return a + b;
}

#[test]
fn test_addition() {
    assert_eq!(3, addition(1, 2));
}