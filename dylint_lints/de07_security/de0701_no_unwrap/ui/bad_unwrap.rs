// Test file for DE0701: No Unwrap Usage
#![allow(dead_code)]

fn bad_examples() {
    let opt: Option<i32> = Some(42);
    // Should trigger DE0701 - no unwrap
    let _val = opt.unwrap();
    
    let res: Result<i32, &str> = Ok(42);
    // Should trigger DE0701 - no unwrap
    let _val2 = res.unwrap();
}

fn main() {}
