#[derive(Serialize, Deserialize, Debug)]
pub struct LoginUserInput {
    pub email: String,
    pub password: String,
}