use std::fmt;

use peregrine_toolkit::serdetools::st_field;
use serde::{de::Visitor, Deserializer, Deserialize};
use serde_repr::{ Serialize_repr, Deserialize_repr };
use crate::request::core::miniresponse::MiniResponseVariety;

#[derive(serde_derive::Deserialize)]
#[serde(transparent)]
pub struct FailureRes {
    message: String
}

impl FailureRes {
    pub fn new(msg: &str) -> FailureRes { FailureRes { message: msg.to_string() } }
    pub fn message(&self) -> &str { &self.message }
}

impl MiniResponseVariety for FailureRes {
    fn description(&self) -> &str { "failure" }
}

#[derive(Serialize_repr,Deserialize_repr,PartialEq,Debug,Clone)]
#[repr(u8)]
pub enum UnavailableReason {
    BadVersion = 0
}

pub struct UnavailableRes(UnavailableReason);

impl UnavailableRes {
    pub fn new(reason: &UnavailableReason) -> UnavailableRes { UnavailableRes(reason.clone()) }
    pub fn reason(&self) -> &UnavailableReason { &self.0 }
}

impl MiniResponseVariety for UnavailableRes {
    fn description(&self) -> &str { "unavailable" }
}

struct UnavailableResVisitor;

impl<'de> Visitor<'de> for UnavailableResVisitor {
    type Value = UnavailableRes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an UnavailableRes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let reason = st_field("reason",seq.next_element()?)?;
        Ok(UnavailableRes(reason))
    }
}

impl<'de> Deserialize<'de> for UnavailableRes {
    fn deserialize<D>(deserializer: D) -> Result<UnavailableRes, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(UnavailableResVisitor)
    }
}
