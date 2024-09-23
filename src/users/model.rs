#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
}

impl User {
    pub fn tranform_to_user_response(&self) -> UserResponse {
        UserResponse {
            id: self.id,
            name: self.name.clone(),
            email: self.email.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserCreateInput {
    pub name: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

impl UserCreateInput {
    pub fn tranform_to_user(&self, hash_password: String) -> User {
        User {
            id: 0,
            name: self.name.clone(),
            email: self.email.clone(),
            password: hash_password,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserUpdateInput {
    pub id: i32,
    pub name: String,
}

impl UserUpdateInput {
    pub fn tranform_to_user(&self) -> User {
        User {
            id: self.id,
            name: self.name.clone(),
            email: "".to_string(),
            password: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
}

pub fn tranform_users_to_user_responses(users: Vec<User>) -> Vec<UserResponse> {
    users.into_iter().map(|user| user.tranform_to_user_response()).collect()
}