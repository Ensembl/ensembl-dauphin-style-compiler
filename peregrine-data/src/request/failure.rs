use std::any::Any;
use anyhow::{ self, };
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_string };
use crate::run::pgcommander::{ PgCommander, PgCommanderTaskSpec };
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use super::request::{ RequestType, ResponseType, ResponseBuilderType };

pub struct GeneralFailure {
    message: String
}

impl GeneralFailure {
    pub fn new(msg: &str) -> GeneralFailure {
        GeneralFailure { message: msg.to_string() }
    }

    pub fn message(&self) -> &str { &self.message }
}

impl ResponseType for GeneralFailure {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct GeneralFailureBuilderType();
impl ResponseBuilderType for GeneralFailureBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        Ok(Box::new(GeneralFailure{
            message: cbor_string(value)?
        }))
    }
}
