use tokio_postgres::Client;
use crate::libs::{ get_id, INTERNAL_ERROR, OK_RESPONSE, NOT_FOUND };
use super::super::repository::get_user_by_id;
use log::error;

pub async fn handle(request: &str, db: &Client) -> (String, String) {
    match get_id(&request).parse::<i32>() {
        Ok(id) => {
            let user = match get_user_by_id(&id, db).await {
                Ok(user) => user,
                _ => return (NOT_FOUND.to_string(), "User not found".to_string()),
            };

            let user_response = user.tranform_to_user_response(); 
            match serde_json::to_string(&user_response) {
                Ok(user) => (OK_RESPONSE.to_string(), user),
                Err(e) => {
                    error!("Error serializing user: {:?}", e);
                    (INTERNAL_ERROR.to_string(), "Internal error".to_string())
                }
            } 
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}