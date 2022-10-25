use serde::{Deserialize, Serialize};

pub mod api;
pub mod markdown;
pub mod subprocess;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub country_code: String,
    pub username: String,
}

pub enum Field {
    Flag,
    Username,
}

impl User {
    pub fn valid(&self, mention: &markdown::UserMention, country_required: bool, name_required: bool) -> bool {
        (!name_required || self.username == mention.username.text) && {
            if let Some(country) = &mention.country_code {
                self.country_code == country.text
            } else {
                !country_required
            }
    }
}
}
