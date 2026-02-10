use modkit_macros::domain_model;

#[domain_model]
pub struct SettingsFields;

impl SettingsFields {
    pub const THEME: &'static str = "theme";
    pub const LANGUAGE: &'static str = "language";
}
