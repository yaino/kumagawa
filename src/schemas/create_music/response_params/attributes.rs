use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Attributes {
    pub timestamp: String,
}
