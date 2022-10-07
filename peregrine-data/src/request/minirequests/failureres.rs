use serde_derive::Deserialize;

use crate::request::core::response::MiniResponseVariety;

#[derive(Deserialize)]
#[serde(transparent)]
pub struct FailureRes {
    message: String
}

impl FailureRes {
    pub fn new(msg: &str) -> FailureRes {
        FailureRes { message: msg.to_string() }
    }

    pub fn message(&self) -> &str { &self.message }
}


impl MiniResponseVariety for FailureRes {
    fn description(&self) -> &str { "failure" }
}
