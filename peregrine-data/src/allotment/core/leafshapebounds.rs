use crate::allotment::util::rangeused::RangeUsed;

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct LeafShapeBounds {
    base_range: RangeUsed<f64>,
    pixel_range: RangeUsed<f64>,
    max_y: f64,
}

impl LeafShapeBounds {
    pub(crate) fn new() -> LeafShapeBounds {
        LeafShapeBounds {
            base_range: RangeUsed::None,
            pixel_range: RangeUsed::None,
            max_y: 0.
        }
    }

    pub(crate) fn base_range(&self) -> &RangeUsed<f64> { &self.base_range }
    pub(crate) fn pixel_range(&self) -> &RangeUsed<f64> { &self.pixel_range }
    pub(crate) fn max_y(&self) -> f64 { self.max_y }

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