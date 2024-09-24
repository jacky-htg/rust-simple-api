use tokio_postgres::Client;
use log::error;
use crate::libs::{ INTERNAL_ERROR, OK_RESPONSE, BAD_REQUEST };
use crate::users::model::UserCreateInput;
use super::super::repository::{get_user_by_id, insert_user, is_email_exist};
use super::util::get_user_create_input;
use super::super::model::User;
use bcrypt;
use regex::Regex;

pub async fn handle(request: &str, db: &mut Client) -> (String, String) {
    match get_user_create_input(&request) {
        Ok(user) => {
            match validate(&user, db).await {
                Ok(_) => (),
                Err(e) => return (BAD_REQUEST.to_string(), e.to_string()),
                
            }
            
            let hash_password = match bcrypt::hash(user.password.clone(), bcrypt::DEFAULT_COST) {
                Ok(hash_password) => hash_password,
                Err(e) => {
                    error!("Error hashing password: {:?}", e);
                    return (INTERNAL_ERROR.to_string(), "Internal error".to_string());
                }
            };

            let user = user.tranform_to_user(hash_password);
            let tx = match db.transaction().await {
                Ok(tx) => tx,
                Err(e) => {
                    error!("Failed to start transaction: {:?}", e);
                    return (INTERNAL_ERROR.to_string(), "Failed to start transaction".to_string());
                }
            };        

            let user: User = match insert_user(&user, &tx).await {
                Ok(user) => {
                    if let Err(e) = tx.commit().await {
                        error!("Failed to commit transaction: {:?}", e);
                        return (INTERNAL_ERROR.to_string(), "Failed to commit transaction".to_string());
                    }
                    user
                }
                Err(e) => {
                    error!("Error creating user: {:?}", e);
                    return (INTERNAL_ERROR.to_string(), "Failed to create new user".to_string());
                }
            };

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
        Err(msg) => {
            error!("Error creating user: {}", msg);
            return (INTERNAL_ERROR.to_string(), msg);
        }
    }
}

async fn validate(user: &UserCreateInput, db: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    if user.name.is_empty() || user.email.is_empty() || user.password.is_empty() {
        return Err("Missing name or email or password".into());
    }

    let email_regex = Regex::new(r"^[^@]+@[^@]+\.[^@]+$")?;
    if !email_regex.is_match(&user.email) {
        return Err("Invalid email format".into());
    }

    if user.password != user.confirm_password {
        return Err("Passwords do not match".into())
    }

    if user.password.len() < 10 {
        return Err("Password must be at least 8 characters long".into());
    }
    
    let has_uppercase = user.password.chars().any(|c| c.is_uppercase());
    let has_lowercase = user.password.chars().any(|c| c.is_lowercase());
    let has_digit = user.password.chars().any(|c| c.is_digit(10));
    let has_special = user.password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase || !has_lowercase || !has_digit || !has_special {
        return Err("Password must contain uppercase, lowercase, digit, and special character".into());
    }

    if user.name.len() < 2 {
        return Err("Name must be at least 2 characters long".into());
    }

    match is_email_exist(&user.email, db).await {
        Ok(is_exist) => if is_exist {
            return Err( "Email already exists".into())   
        }
        Err(e) => {
            error!("Error checking if email already exists: {:?}", e);
            return Err("Internal error".into());
        }  
    }

    Ok(())
}