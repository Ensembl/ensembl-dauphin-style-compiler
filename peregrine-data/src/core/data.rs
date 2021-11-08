use serde::Deserializer;
use serde::de::Visitor;
use std::fmt;
use std::sync::Arc;

#[derive(Clone)]
pub struct ReceivedData(Arc<Vec<u8>>);

impl ReceivedData {
    pub fn len(&self) -> usize { self.0.len() }
    pub fn data(&self) -> &Arc<Vec<u8>> { &self.0 }
}

struct DataVisitor;

impl<'de> Visitor<'de> for DataVisitor {
    type Value = ReceivedData;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"some data") }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<ReceivedData,E> where E: serde::de::Error {
        Ok(ReceivedData(Arc::new(v.to_vec())))
    }
}

impl<'de> serde::Deserialize<'de> for ReceivedData {
    fn deserialize<D>(deserializer: D) -> Result<ReceivedData, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_bytes(DataVisitor)
    }
}
