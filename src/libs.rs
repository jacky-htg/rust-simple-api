pub mod token;
use anyhow::{Error, Result};
use std::env;

pub const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
pub const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
pub const NO_CONTENT: &str = "HTTP/1.1 204 NO CONTENT\r\n\r\n";
pub const BAD_REQUEST: &str = "HTTP/1.1 400 BAD REQUEST\r\n\r\n";
pub const UNAUTHORIZED: &str = "HTTP/1.1 401 UNAUTHORIZED\r\n\r\n";
pub const INTERNAL_ERROR: &str = "HTTP/1.1 500 INTERNAL ERROR\r\n\r\n";
pub const TOO_MANY_REQUEST: &str = "HTTP/1.1 429 TOO MANY REQUESTS\r\n\r\n";
pub const CORS_ALLOW_ALL: &str = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";

//Get id from request URL
pub fn get_id(request: &str) -> &str {
    request
        .split("/")
        .nth(2)
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or_default()
}

pub fn get_db_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

pub fn get_worker_num() -> usize {
    env::var("WORKER_NUM")
        .unwrap_or_else(|_| "2".to_string())
        .parse()
        .expect("WORKER_NUM must be a number")
}

pub async fn authenticate(request: &str) -> Result<String, Error> {
    let token = request
        .split("\r\n")
        .find(|s| s.starts_with("Authorization: Bearer "))
        .and_then(|s| s.split_whitespace().nth(2))
        .ok_or_else(|| Error::msg("Authorization header not found"))?;
    match token::validate_token(token) {
        Ok(email) => Ok(email),
        Err(e) => Err(anyhow::Error::msg(e.to_string())), // Ubah ke tipe error yang mendukung Send + Sync
    }
}
