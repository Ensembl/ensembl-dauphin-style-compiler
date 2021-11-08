use serde::Deserializer;
use crate::Stick;

pub struct StickCommandResponse {
    stick: Stick
}

impl StickCommandResponse {
    pub(crate) fn stick(&self) -> Stick { self.stick.clone() }
}

impl<'de> serde::Deserialize<'de> for StickCommandResponse {
    fn deserialize<D>(deserializer: D) -> Result<StickCommandResponse, D::Error> where D: Deserializer<'de> {
        Ok(StickCommandResponse {
            stick: Stick::deserialize(deserializer)?
        })
    }
}
