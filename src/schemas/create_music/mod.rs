use serde::{Deserialize, Serialize};

mod request_params;
pub use request_params::{RequestParams, Attributes, Prompt};

mod response_params;
pub use response_params::ResponseParams;

#[derive(Serialize)]
pub struct CreateMusicRequest {
    pub data: RequestParams,
}

#[derive(Deserialize, Clone)]
pub struct CreateMusicResponse {
    pub data: Vec<ResponseParams>,
}
