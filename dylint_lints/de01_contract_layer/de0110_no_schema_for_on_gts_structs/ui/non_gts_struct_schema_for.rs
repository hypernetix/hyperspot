//! Test case: schema_for! on regular (non-GTS) struct should NOT trigger DE0110

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A regular struct (NOT GTS-wrapped, no struct_to_gts_schema attribute)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegularDto {
    pub id: String,
    pub name: String,
}

fn main() {
    // Should not trigger DE0110 - schema_for on non-GTS struct
    let _schema = schemars::schema_for!(RegularDto);
}
