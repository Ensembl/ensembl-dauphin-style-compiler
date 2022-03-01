use peregrine_toolkit::lock;

use crate::allotment::core::rangeused::RangeUsed;

#[derive(Clone)]
pub struct DrawingInfo {
    base_range: RangeUsed<f64>,
    pixel_range: RangeUsed<f64>,
    max_y: f64,
}

impl DrawingInfo {
    pub(crate) fn new() -> DrawingInfo {
        DrawingInfo {
            base_range: RangeUsed::None,
            pixel_range: RangeUsed::None,
            max_y: 0.
        }
    }

    pub fn base_range(&self) -> &RangeUsed<f64> { &self.base_range }
    pub fn pixel_range(&self) -> &RangeUsed<f64> { &self.pixel_range }
    pub fn max_y(&self) -> f64 { self.max_y }

    pub fn merge_max_y(&mut self, new_max: f64) { 
        self.max_y = self.max_y.max(new_max);
    }

    pub fn merge_base_range(&mut self, new_range: &RangeUsed<f64>) {
        self.base_range = self.base_range.merge(new_range);
    }

    pub fn merge_pixel_range(&mut self, new_range: &RangeUsed<f64>) {
        self.pixel_range = self.pixel_range.merge(new_range);
    }
}
