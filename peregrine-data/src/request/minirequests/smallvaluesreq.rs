use serde::{Serialize, ser::SerializeSeq};
use crate::{MiniRequest, request::core::minirequest::MiniRequestVariety};

pub struct SmallValuesReq {
    namespace: String,
    column: String
}

impl SmallValuesReq {
    pub(crate) fn new(namespace: &str, column: &str) -> MiniRequest {
        MiniRequest::SmallValues(SmallValuesReq {
            namespace: namespace.to_string(),
            column: column.to_string()
        })
    }
}

impl MiniRequestVariety for SmallValuesReq {
    fn description(&self) -> String { "small-values".to_string() }
    fn opcode(&self) -> u8 { 8 }
}

impl Serialize for SmallValuesReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.namespace)?;
        seq.serialize_element(&self.column)?;
        seq.end()
    }
}
