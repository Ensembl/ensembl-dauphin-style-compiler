use std::{sync::Arc};

#[derive(Clone)]
pub struct ReceivedData(Arc<Vec<u8>>);

impl ReceivedData {
    pub fn new(data: Vec<u8>) -> ReceivedData { ReceivedData(Arc::new(data)) }

    pub fn len(&self) -> usize { self.0.len() }
    pub fn data(&self) -> &Arc<Vec<u8>> { &self.0 }
}
