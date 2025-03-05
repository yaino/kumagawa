use serde::Serialize;

#[derive(Serialize)]
pub struct Prompt {
    pub text: String,
    pub weight: u32,
}
