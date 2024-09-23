use tokio_postgres::Client;
use super::util::get_user_update_input;
use crate::libs::{ get_id, BAD_REQUEST, INTERNAL_ERROR, NOT_FOUND, OK_RESPONSE };
use super::super::repository::{update_user, get_user_by_id};
use log::error;
use super::super::model::UserUpdateInput;

pub async fn handle(request: &str, db: &Client) -> (String, String) {
    match
        (
            get_id(&request).parse::<i32>(),
            get_user_update_input(&request),
        )
    {
        (Ok(id), Ok(user)) => {
            match validate(id, &user).await {
                Ok(_) => (),
                Err(e) => return (BAD_REQUEST.to_string(), e.to_string()),
            }

            let user = user.tranform_to_user();
            match get_user_by_id(&user.id, db).await {
                Ok(_) => {},
                Err(e) => {
                    error!("Error getting user with id '{}': {}", user.id, e);
                    return (NOT_FOUND.to_string(), "User not found".to_string())
                }
            }

            match update_user(&user, db).await {
                Ok(_) => {},
                Err(e) => {
                    error!("Error updating user with id '{}': {}", user.id, e);
                    return (INTERNAL_ERROR.to_string(), "Failed to update user".to_string())
                }
            }
            
            let user = match get_user_by_id(&user.id, db).await {
                Ok(user) => user,
                _ => {
                    error!("Error getting user with id '{}'", user.id);
                    return (INTERNAL_ERROR.to_string(), "Internal error".to_string())
                }
            };

            let user = user.tranform_to_user_response();
            match serde_json::to_string(&user) {
                Ok(user) => (OK_RESPONSE.to_string(), user),
                Err(e) => {
                    error!("Error serializing user: {:?}", e);
                    (INTERNAL_ERROR.to_string(), "Internal error".to_string())
                }
            }
        }
        _ => {
            error!("Error updating user");
            (INTERNAL_ERROR.to_string(), "Internal error".to_string())
        }
    }
}

async fn validate(id: i32, user: &UserUpdateInput) -> Result<(), Box<dyn std::error::Error>> {
    if id != user.id {
        return Err("User id in path does not match user id in body".into())
    }

    if user.name.is_empty() {
        return Err("Missing name".into());
    }

    if user.name.len() < 2 {
        return Err("Name must be at least 2 characters long".into());
    }

    Ok(())
}