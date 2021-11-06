use std::fmt;
use serde::{Deserialize, Deserializer, de::Visitor};

pub struct GeneralFailure {
    message: String
}

impl GeneralFailure {
    pub fn new(msg: &str) -> GeneralFailure {
        GeneralFailure { message: msg.to_string() }
    }

    pub fn message(&self) -> &str { &self.message }
}

struct GeneralFailureVisitor;

impl<'de> Visitor<'de> for GeneralFailureVisitor {
    type Value = GeneralFailure;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"a general failure") }

    fn visit_string<E>(self, v: String) -> Result<GeneralFailure, E> where E: serde::de::Error {
        Ok(GeneralFailure { message: v })
    }
}

impl<'de> Deserialize<'de> for GeneralFailure {
    fn deserialize<D>(deserializer: D) -> Result<GeneralFailure, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_string(GeneralFailureVisitor)
    }
}
