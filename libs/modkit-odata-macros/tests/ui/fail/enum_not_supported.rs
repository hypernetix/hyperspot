use modkit_odata_macros::ODataSchema;

#[derive(ODataSchema)]
enum Status {
    Active,
    Inactive,
}

fn main() {}
