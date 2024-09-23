use tokio_postgres::Client;
use super::super::repository::list_users;
use crate::libs::{INTERNAL_ERROR, OK_RESPONSE};
use super::super::model::tranform_users_to_user_responses;

pub async fn handle(_request: &str, client: &Client) -> (String, String) {
    let users = match list_users(client).await {
        Ok(users) => users,
        _ => return (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    };

    let user_responses = tranform_users_to_user_responses(users);
    match serde_json::to_string(&user_responses) {
        Ok(user_responses) => (OK_RESPONSE.to_string(), user_responses),  
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}
