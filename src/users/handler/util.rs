use super::super::model::{UserUpdateInput, UserCreateInput};

pub fn get_user_create_input(request: &str) -> Result<UserCreateInput, String> {
    let body = request.split("\r\n\r\n").last().unwrap_or_default();
    serde_json::from_str(body).map_err(|e| {
        format!("Failed to parse request body: {}", e)
    })
}

pub fn get_user_update_input(request: &str) -> Result<UserUpdateInput, String> {
    let body = request.split("\r\n\r\n").last().unwrap_or_default();
    serde_json::from_str(body).map_err(|e| {
        format!("Failed to parse request body: {}", e)
    })
}