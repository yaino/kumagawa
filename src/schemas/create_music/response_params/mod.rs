use serde::Deserialize;

mod attributes;
pub use attributes::Attributes;

mod music_links;
pub use music_links::MusicLinks;

#[derive(Deserialize, Clone)]
pub struct ResponseParams {
    pub r#type: String,
    pub id: String,
    pub attributes: Attributes,
    pub links: MusicLinks,
}
