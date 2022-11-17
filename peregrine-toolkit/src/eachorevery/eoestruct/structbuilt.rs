use std::fmt;
use std::{sync::Arc, collections::hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use serde::de::{Visitor, MapAccess};
use serde::{Deserializer, Deserialize};
use super::eoestruct::{StructConst, StructVarValue};

#[derive(Clone)]
pub enum StructBuilt {
    Var(usize,usize),
    Const(StructConst),
    Array(Arc<Vec<StructBuilt>>,bool),
    Object(Arc<Vec<(String,StructBuilt)>>),
    All(Vec<Option<Arc<StructVarValue>>>,Arc<StructBuilt>),
    Condition(usize,usize,Arc<StructBuilt>)
}

impl StructBuilt {
    pub fn is_null(&self) -> bool {
        if let StructBuilt::Const(StructConst::Null) = self { true } else { false }
    }
}


macro_rules! sb_ds_number {
    ($name:ident,$type:ty) => {
        fn $name<E>(self, v: $type) -> Result<Self::Value, E> where E: serde::de::Error {
            Ok(StructBuilt::Const(StructConst::Number(v as f64)))
        }    
    };
}
