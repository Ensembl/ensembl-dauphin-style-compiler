use peregrine_toolkit::cbor::cbor_into_bytes;
use std::sync::Arc;
use serde_cbor::Value as CborValue;

#[derive(Clone)]
pub struct ReceivedData(Arc<Vec<u8>>);

impl ReceivedData {
    pub fn len(&self) -> usize { self.0.len() }
    pub fn data(&self) -> &Arc<Vec<u8>> { &self.0 }

    pub fn decode(value: CborValue) -> Result<ReceivedData,String> {
        Ok(ReceivedData(Arc::new(cbor_into_bytes(value)?)))
    }
}
