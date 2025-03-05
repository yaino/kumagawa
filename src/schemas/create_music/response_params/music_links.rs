use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct MusicLinks {
    pub result: String,
    #[serde(rename = "self")]
    pub endpoint: String,
}
