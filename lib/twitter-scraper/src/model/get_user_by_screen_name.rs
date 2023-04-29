use super::{
    ResultField,
    UserField,
};

pub type GetUserByScreenNameResponse = UserField<ResultField<UserResult>>;

#[derive(Debug, serde::Deserialize)]
pub struct UserResult {
    pub rest_id: String,
}
