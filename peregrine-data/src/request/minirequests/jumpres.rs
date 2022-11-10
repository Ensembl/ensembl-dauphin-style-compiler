use std::fmt;
use peregrine_toolkit::serdetools::st_field;
use serde::{Deserializer, Deserialize, de::{Visitor, MapAccess, IgnoredAny}};

use crate::request::core::response::MiniResponseVariety;

pub struct JumpLocation {
    pub stick: String,
    pub left: u64,
    pub right: u64
}

pub enum JumpRes {
    Found(JumpLocation),
    NotFound
}

struct JumpVisitor;

impl<'de> Visitor<'de> for JumpVisitor {
    type Value = JumpRes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JumpRes")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut stick = None;
        let mut left = None;
        let mut right = None;
        let mut found = true;
        while let Some(key) = access.next_key()? {
            match key {
                "stick" => { stick = access.next_value()? },
                "left" => { left = access.next_value()? },
                "right" => { right = access.next_value()? },
                "no" => { let _ : IgnoredAny = access.next_value()?; found = false; }
                _ => {}
            }
        }
        if found {
            let stick = st_field("stick",stick)?;
            let left = st_field("left",left)?;
            let right = st_field("right",right)?;    
            Ok(JumpRes::Found(JumpLocation{ stick, left, right }))
        } else {
            Ok(JumpRes::NotFound)
        }
    }
}

impl<'de> Deserialize<'de> for JumpRes {
    fn deserialize<D>(deserializer: D) -> Result<JumpRes, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(JumpVisitor)
    }
}

impl MiniResponseVariety for JumpRes {
    fn description(&self) -> &str { "jump" }
}
