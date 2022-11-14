use std::fmt;
use peregrine_toolkit::{serdetools::st_field};
use serde::{Deserialize, Deserializer, de::{MapAccess, Visitor}};
use crate::{Stick, request::core::miniresponse::MiniResponseVariety};

#[derive(Clone)]
pub enum StickRes {
    Stick(Stick),
    Unknown(String)
}

struct StickVisitor;

impl<'de> Visitor<'de> for StickVisitor {
    type Value = StickRes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a StickRes")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut id = None;
        let mut size = None;
        let mut topology = None;
        let mut tags = None;
        let mut error = None;
        while let Some(key) = access.next_key()? {
            match key {
                "id" => { id = Some(access.next_value()?); },
                "size" => { size = Some(access.next_value()?); },
                "topology" => { topology = Some(access.next_value()?); },
                "tags" => { tags = Some(access.next_value()?); },
                "error" => { error = Some(access.next_value()?); }
                _ => {}
            }
        }
        if let Some(error) = error {
            Ok(StickRes::Unknown(error))
        } else {
            let id = st_field("id",id)?;
            let size = st_field("size",size)?;
            let topology = st_field("topology",topology)?;
            let tags : Vec<_> = st_field("tags",tags)?;
            Ok(StickRes::Stick(Stick::new(&id,size,topology,&tags)))
        }
    }
}

impl<'de> Deserialize<'de> for StickRes {
    fn deserialize<D>(deserializer: D) -> Result<StickRes, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(StickVisitor)
    }
}

impl MiniResponseVariety for StickRes {
    fn description(&self) -> &str { "stick" }
}
