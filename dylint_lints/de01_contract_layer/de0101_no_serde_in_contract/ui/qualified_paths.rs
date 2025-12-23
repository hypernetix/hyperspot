// simulated_dir=/hyperspot/modules/some_module/contract/
#[allow(dead_code)]
// Should trigger DE0101 - Serde in contract
#[derive(Debug, Clone, serde::Serialize)]
pub struct WithQualifiedSerialize {
    pub id: String,
}

#[allow(dead_code)]
// Should trigger DE0101 - Serde in contract
#[derive(Debug, serde::Deserialize)]
pub struct WithQualifiedDeserialize {
    pub id: String,
}

#[allow(dead_code)]
// Should trigger DE0101 - Serde in contract
#[derive(serde::Serialize, serde::Deserialize)]
pub struct WithBothQualified {
    pub id: String,
}

fn main() {}
