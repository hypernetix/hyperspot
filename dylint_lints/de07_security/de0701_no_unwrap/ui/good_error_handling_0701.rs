// Test file for DE0701: No Unwrap Usage - Good Examples
#![allow(dead_code)]

fn good_examples() -> Result<(), &'static str> {
    let opt: Option<i32> = Some(42);
    // Should not trigger DE0701 - no unwrap
    let _val = opt.ok_or("missing value")?;
    
    let res: Result<i32, &str> = Ok(42);
    // Should not trigger DE0701 - no unwrap
    let _val2 = res?;
    
    // Match is also fine
    let opt2: Option<i32> = Some(10);
    // Should not trigger DE0701 - no unwrap
    let _val3 = match opt2 {
        Some(v) => v,
        None => return Err("not found"),
    };
    
    Ok(())
}

fn main() {}
