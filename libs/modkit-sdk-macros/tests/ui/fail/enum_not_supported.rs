use modkit_sdk_macros::ODataSchema;

#[derive(ODataSchema)]
enum Status {
    Active,
    Inactive,
}

fn main() {}
