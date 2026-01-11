// Test file for DE0702: no expect usage

fn main() {
    let option: Option<i32> = Some(42);
    // Should trigger DE0702 - no expect
    let _value = option.expect("should have value");

    let result: Result<i32, &str> = Ok(42);
    // Should trigger DE0702 - no expect
    let _data = result.expect("should succeed");
}
