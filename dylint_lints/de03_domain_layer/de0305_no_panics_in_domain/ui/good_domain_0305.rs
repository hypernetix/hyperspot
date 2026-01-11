// simulated_dir=/hyperspot/modules/some_module/domain/
// Test file for DE0305: proper error handling in domain (no panics)

#[derive(Debug)]
enum DomainError {
    EmptyInput,
    InvalidValue,
    NotImplemented(&'static str),
}

// Should not trigger DE0305 - panic in domain
fn validate_input(data: &str) -> Result<(), DomainError> {
    if data.is_empty() {
        return Err(DomainError::EmptyInput);
    }
    Ok(())
}

// Should not trigger DE0305 - panic in domain
fn process_data() -> Result<(), DomainError> {
    Err(DomainError::NotImplemented("data processing"))
}

// Should not trigger DE0305 - panic in domain
fn handle_case(value: i32) -> Result<(), DomainError> {
    match value {
        1 => Ok(()),
        _ => Err(DomainError::InvalidValue),
    }
}

// Should not trigger DE0305 - panic in domain
fn calculate() -> Result<i32, DomainError> {
    Err(DomainError::NotImplemented("calculation"))
}

fn main() {
    let _ = validate_input("");
    let _ = process_data();
    let _ = handle_case(1);
    let _ = calculate();
}
