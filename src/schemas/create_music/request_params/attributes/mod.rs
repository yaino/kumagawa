mod prompt;
pub use prompt::Prompt;

use serde::Serialize;

#[derive(Serialize)]
pub struct Attributes {
    pub prompts: Vec<Prompt>,
    pub length_seconds: u32,
    pub seed: u32,
}
