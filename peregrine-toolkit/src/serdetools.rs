use std::fmt;

use serde::{Deserializer, de::Visitor, Deserialize};

pub fn st_field<T,E>(name: &'static str, value: Option<T>) -> Result<T,E> where E: serde::de::Error {
    value.ok_or(serde::de::Error::missing_field(name))
}

pub fn st_err<T,E,F>(value: Result<T,F>,text: &str) -> Result<T,E> where E: serde::de::Error, F: std::fmt::Debug {
    value.map_err(|e| serde::de::Error::custom(format!("{}: {:?}",text,e)))
}

#[derive(serde_derive::Deserialize)]
#[serde(transparent)]
pub struct ByteData {
    #[serde(with="serde_bytes")]
    pub data: Vec<u8>
}

macro_rules! force_string {
    ($name:ident,$type:ty,$convert:expr) => {
        fn $name<E>(self, v: $type) -> Result<Self::Value, E> where E: serde::de::Error {
            Ok(ForceString(($convert)(v).to_string()))
        }    
    };

    ($name:ident,$type:ty) => {
        fn $name<E>(self, v: $type) -> Result<Self::Value, E> where E: serde::de::Error {
            Ok(ForceString(v.to_string()))
        }
    };

    ($name:ident) => {
        fn $name<E>(self) -> Result<Self::Value, E> where E: serde::de::Error {
            Ok(ForceString("".to_string()))
        }
    };
}

pub struct ForceString(pub String);

struct ForceStringVisitor;

impl<'de> Visitor<'de> for ForceStringVisitor {
    type Value = ForceString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an atom coercable into a string")
    }

    force_string!(visit_bool,bool,(|v| if v { "true " } else { "" }));
    force_string!(visit_i64,i64);
    force_string!(visit_i128,i128);
    force_string!(visit_u64,u64);
    force_string!(visit_u128,u128);
    force_string!(visit_f64,f64);
    force_string!(visit_str,&str);
    force_string!(visit_none);
    force_string!(visit_unit);
}

impl<'de> Deserialize<'de> for ForceString {
    fn deserialize<D>(deserializer: D) -> Result<ForceString, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_string(ForceStringVisitor)
    }
}
