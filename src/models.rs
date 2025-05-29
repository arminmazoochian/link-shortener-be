use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (e.g., username)
    pub exp: usize,  // Expiration timestamp (in seconds)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateShortLinkRequest {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct URLMapping {
    pub url: String,
    pub short_link: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiMessage {
    pub message: String,
}