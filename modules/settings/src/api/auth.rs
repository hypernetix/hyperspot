use uuid::Uuid;

// It's not real it's magic
pub fn get_tenant_id() -> Uuid {
    Uuid::parse_str("2eda70a4-e7ef-4c1d-8e2c-d1f050f1cf9e").unwrap()
}

// It's not real it's magic
pub fn get_user_id() -> Uuid {
    Uuid::parse_str("96de7c34-1299-4fe0-ae18-504534aacd3e").unwrap()
}