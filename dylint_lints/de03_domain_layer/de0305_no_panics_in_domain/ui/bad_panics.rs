// simulated_dir=/hyperspot/modules/some_module/domain/
// Test file for DE0305: panic macros in domain layer (should trigger)

fn validate_input(data: &str) {
    // Should trigger DE0305 - panic in domain
    panic!("data validation failed");
}

fn process_data() {
    // Should trigger DE0305 - panic in domain
    todo!("implement data processing");
}

fn handle_case(value: i32) {
    match value {
        1 => {}
        // Should trigger DE0305 - panic in domain
        _ => unreachable!(),
    }
}

fn calculate() -> i32 {
    // Should trigger DE0305 - panic in domain
    unimplemented!()
}

fn main() {
    validate_input("");
    process_data();
    handle_case(1);
    let _ = calculate();
}
