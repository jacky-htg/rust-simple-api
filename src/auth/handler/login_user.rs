use tokio_postgres::Client;
use super::super::model::LoginUserInput;
use crate::libs::{BAD_REQUEST, OK_RESPONSE, INTERNAL_ERROR};
use crate::users::repository::get_password_by_email;
use log::error;
use bcrypt;
use crate::libs::token::claim_jwt_token;

pub async fn handle(request: &str, db: &Client) -> (String, String) {
    let login_input: LoginUserInput= match get_user_login_input(request) {
        Ok(login_input) => login_input,
        Err(msg) => return (BAD_REQUEST.to_string(), msg.to_string()),
    };

    match validate(&login_input, db).await {
        Ok(()) => (),
        Err(e) => return (BAD_REQUEST.to_string(), e.to_string()),
    };

    let token = match claim_jwt_token(login_input.email) {
        Ok(token) => token,
        Err(e) => {
            return (INTERNAL_ERROR.to_string(), e.to_string());
        }
    };

    let body = serde_json::json!({
        "token": token
    }).to_string();

    (OK_RESPONSE.to_string(), String::from(body))
}

fn get_user_login_input(request: &str) -> Result<LoginUserInput, String> {
    let body = request.split("\r\n\r\n").last().unwrap_or_default();
    serde_json::from_str(body).map_err(|e| {
        format!("Failed to parse request body: {}", e)
    })
}

async fn validate(user: &LoginUserInput, db: &Client) -> Result<(), Box<dyn std::error::Error>> {
    if user.email.is_empty() || user.password.is_empty() {
        return Err("Invalid email or password".into())
    }

    let password = match get_password_by_email(&user.email, db).await {
        Ok(password) => password.trim_end().to_string(),
        Err(e) => {
            error!("Error getting password: {:?}", e);
            return Err("Invalid email or password".into())
        }
    };

    match bcrypt::verify(&user.password, &password) {
        Ok(_) => (),
        Err(e) => {
            error!("Error verifying password: {:?}", e);
            return Err("Invalid email or password".into())
        }
    }

    Ok(())

}