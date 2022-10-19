use std::fmt;
use std::{sync::Arc, collections::hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use serde::de::{Visitor, MapAccess};
use serde::{Deserializer, Deserialize};

use crate::eachorevery::EachOrEvery;
use super::eoestruct::{StructConst, StructVarValue};

#[derive(Clone)]
pub enum StructBuilt {
    Var(usize,usize),
    Const(StructConst),
    Array(Arc<EachOrEvery<StructBuilt>>,bool),
    Object(Arc<EachOrEvery<(String,StructBuilt)>>),
    All(Vec<Option<Arc<StructVarValue>>>,Arc<StructBuilt>),
    Condition(usize,usize,Arc<StructBuilt>)
}

impl StructBuilt {
    pub fn is_null(&self) -> bool {
        self == &StructBuilt::Const(StructConst::Null)
    }
}

impl Hash for StructBuilt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            StructBuilt::Var(a,b) => { a.hash(state); b.hash(state); },
            StructBuilt::Const(c) => c.hash(state),
            StructBuilt::Array(a, _) => { a.hash(state); },
            StructBuilt::Object(b) => { b.hash(state); },
            StructBuilt::All(v,b) => { v.hash(state); b.hash(state); },
            StructBuilt::Condition(a,b,c) => { a.hash(state); b.hash(state); c.hash(state); },
        }
    }
}

impl PartialEq for StructBuilt {
    fn eq(&self, other: &Self) -> bool {
        let mut self_hash = DefaultHasher::new();
        self.hash(&mut self_hash);
        let mut other_hash = DefaultHasher::new();
        other.hash(&mut other_hash);
        self_hash.finish() == other_hash.finish()
    }
}

impl Eq for StructBuilt {}

macro_rules! sb_ds_number {
    ($name:ident,$type:ty) => {
        fn $name<E>(self, v: $type) -> Result<Self::Value, E> where E: serde::de::Error {
            Ok(StructBuilt::Const(StructConst::Number(v as f64)))
        }    
    };
}

struct StructBuiltVisitor;

impl<'de> Visitor<'de> for StructBuiltVisitor {
    type Value = StructBuilt;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a MiniResponse")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let mut data : Vec<StructBuilt> = vec![];
        while let Some(value) = seq.next_element()? { data.push(value); }
        Ok(StructBuilt::Array(Arc::new(EachOrEvery::each(data)),false))
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut data : Vec<(String,StructBuilt)> = vec![];
        while let Some((key,value)) = access.next_entry()? {
            data.push((key,value));
        }
        Ok(StructBuilt::Object(Arc::new(EachOrEvery::each(data))))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where E: serde::de::Error {
        Ok(StructBuilt::Const(StructConst::Boolean(v)))
    }

    sb_ds_number!(visit_i64,i64);
    sb_ds_number!(visit_i128,i128);
    sb_ds_number!(visit_u64,u64);
    sb_ds_number!(visit_u128,u128);
    sb_ds_number!(visit_f64,f64);

    fn visit_none<E>(self) -> Result<Self::Value, E>
            where E: serde::de::Error {
        Ok(StructBuilt::Const(StructConst::Null))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: serde::de::Error {
        Ok(StructBuilt::Const(StructConst::String(v.to_string())))
    }
}

impl<'de> Deserialize<'de> for StructBuilt {
    fn deserialize<D>(deserializer: D) -> Result<StructBuilt, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(StructBuiltVisitor)
    }
}
