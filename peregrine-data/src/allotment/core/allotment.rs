use std::fmt::Debug;
use std::sync::{Arc};
use crate::allotment::tree::allotmentbox::AllotmentBox;
use crate::AllotmentMetadataRequest;

pub trait Transformer {
    fn add_transform_metadata(&self, out: &mut AllotmentMetadataRequest);
}

#[derive(Clone)]
pub struct Allotment(Arc<dyn Transformer>,Arc<AllotmentBox>);

#[cfg(debug_assertions)]
impl Debug for Allotment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Allotment(...)")
    }
}

impl Allotment {
    pub fn new(allotment_impl: Arc<dyn Transformer>, allot_box: Arc<AllotmentBox>) -> Allotment {
        Allotment(allotment_impl,allot_box)
    }

    pub fn allotment_box(&self) -> &Arc<AllotmentBox> { &self.1 }
}
