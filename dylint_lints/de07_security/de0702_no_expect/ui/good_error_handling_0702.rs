// Test file for DE0702: proper error handling (no expect)

fn process() -> Result<i32, &'static str> {
    let option: Option<i32> = Some(42);
    // Should not trigger - using ok_or with ?
    let value = option.ok_or("not found")?;

    let result: Result<i32, &str> = Ok(42);
    // Should not trigger - using map_err with ?
    let data = result.map_err(|e| e)?;

    Ok(value + data)
}

fn main() {
    let _ = process();
}
