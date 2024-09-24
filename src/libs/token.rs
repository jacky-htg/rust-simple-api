use chrono;
use jsonwebtoken::{encode, Header, EncodingKey, decode, DecodingKey, Validation};
use log::error;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
   email: String,
   exp: i64,
}

pub fn claim_jwt_token(email: String) -> Result<String, Box<dyn std::error::Error>> {
    let expiration = chrono::Utc::now() + chrono::Duration::hours(1);
    let claims = Claims {
        email: email,
        exp: expiration.timestamp(),
    };

    let secret_key = match std::env::var("SECRET_KEY") {
        Ok(secret_key) => secret_key,
        Err(_) => {
            error!("SECRET_KEY environment variable is not set");
            return Err("SECRET_KEY environment variable is not set".into())
        }
    };

    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret_key.as_bytes())) {
        Ok(token) => token,
        Err(e) => {
            error!("Error generating jwt token: {:?}", e);
            return Err("Error generating jwt token".into())
        }
    };
    Ok(token)
}

pub fn validate_token(token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let secret_key = match std::env::var("SECRET_KEY") {
        Ok(secret_key) => secret_key,
        Err(_) => {
            error!("SECRET_KEY environment variable is not set");
            return Err("SECRET_KEY environment variable is not set".into())
        }
    };
    match decode::<Claims>(token, &DecodingKey::from_secret(secret_key.as_bytes()), &Validation::default()) {
        Ok(token_data) => Ok(token_data.claims.email),
        Err(e) => {
            error!("Error validating jwt token: {:?}", e);
            Err("Error validating jwt token".into())
        }
    }
}