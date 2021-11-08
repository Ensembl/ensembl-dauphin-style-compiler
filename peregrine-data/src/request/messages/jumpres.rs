use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct JumpLocation {
    pub stick: String,
    pub left: u64,
    pub right: u64
}

#[derive(Deserialize)]
pub struct NotFound { 
    #[allow(unused)]
    no: bool
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum JumpResponse {
    Found(JumpLocation),
    NotFound(NotFound)
}
