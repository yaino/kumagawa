use serde::Serialize;

mod attributes;
pub use attributes::{Attributes, Prompt};

#[derive(Serialize)]
pub struct RequestParams {
    pub r#type: String,
    pub attributes: Attributes,
}
