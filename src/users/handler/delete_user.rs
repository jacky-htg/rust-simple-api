use tokio_postgres::Client;
use log::error;
use crate::libs::{ get_id, INTERNAL_ERROR, NO_CONTENT, NOT_FOUND };
use super::super::repository::{delete_user_by_id, get_user_by_id};

pub async fn handle(request: &str, db: &Client) -> (String, String) {
    match get_id(&request).parse::<i32>() {
        Ok(id) => {
            match get_user_by_id(&id, db).await  {
                Ok(_) => {},
                Err(e) => {
                    error!("Error getting user with id '{}': {}", id, e);
                    return (NOT_FOUND.to_string(), "User not found".to_string())
                }
            }
            match delete_user_by_id(&id, db).await {
                Ok(_) => (NO_CONTENT.to_string(), "".to_string()),
                Err(e) => {
                    error!("Error deleting user with id '{}': {}", id, e);
                    (INTERNAL_ERROR.to_string(), "Failed to delete user".to_string())
                }
                
            }
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}