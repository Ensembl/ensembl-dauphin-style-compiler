use crate::{allotment::transformers::drawinginfo::DrawingInfo};
use super::allotmentname::AllotmentName;

pub struct PendingLeaf {
    name: AllotmentName,
    drawing_info: DrawingInfo
}

impl PendingLeaf {
    pub fn new(name: &AllotmentName) -> PendingLeaf {
        PendingLeaf {
            name: name.clone(),
            drawing_info: DrawingInfo::new()
        }
    }

    pub fn name(&self) -> &AllotmentName {&self.name }
    pub fn drawing_info(&self) -> &DrawingInfo { &self.drawing_info }
    pub fn drawing_info_mut(&mut self) -> &mut DrawingInfo { &mut self.drawing_info }
}
