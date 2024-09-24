use log::info;
use tokio_postgres::{Client, Error, Transaction};
use super::model::User;

pub async fn insert_user<'a>(user: &User, tx: &'a Transaction<'a>) -> Result<User, Error>{  
    let row = tx.query_one(
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING id",
        &[&user.name, &user.email, &user.password],
    ).await?;
    
    let id: i32 = row.get(0);
    info!("User created: {:?}", id);
    Ok(User {
        id,
        name: user.name.clone(),
        email: user.email.clone(),
        password: "".to_string(),
    })
}

pub async fn get_user_by_id(id: &i32, db: &Client) -> Result<User, Error>{  
    let row = db.query_one("SELECT id, name, email FROM users WHERE id = $1", &[id]).await?;
    Ok(User {
        id: row.get(0),
        name: row.get(1),
        email: row.get(2),
        password: "".to_string(),
    })
}

pub async fn get_password_by_email(email: &str, db: &Client) -> Result<String, Error>{  
    let row = db.query_one("SELECT password FROM users WHERE email = $1", &[&email]).await?;
    Ok(row.get(0))
}

pub async fn is_email_exist(email: &str, db: &Client) -> Result<bool, Error>{  
    let row = db.query_opt("SELECT 1 FROM users WHERE email = $1", &[&email]).await?;
    Ok(row.is_some())
}

pub async fn delete_user_by_id(id: &i32, db: &Client) -> Result<u64, Error>{
    let rows_affected = db.execute("DELETE FROM users WHERE id = $1", &[&id]).await?;
    Ok(rows_affected)
}

pub async fn update_user(user: &User, db: &Client) -> Result<u64, Error> {
    let rows_affected = db.execute("UPDATE users SET name = $1 WHERE id = $2", &[&user.name, &user.id]).await?;
    Ok(rows_affected)
}

pub async fn list_users(db: &Client) -> Result<Vec<User>, Error> {
    let rows = db.query("SELECT id, name, email FROM users", &[]).await?;
    let mut users = Vec::new();
    for row in rows {
        users.push(User {
            id: row.get(0),
            name: row.get(1),
            email: row.get(2),
            password: "".to_string(),
        });
    }
    Ok(users)
}
    