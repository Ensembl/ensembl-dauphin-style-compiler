use crate::{shape::metadata::AllotmentMetadataEntry, allotment::core::rangeused::RangeUsed};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct LeafRequestSize {
    base_range: RangeUsed<f64>,
    pixel_range: RangeUsed<f64>,
    height: f64,
    metadata: Vec<AllotmentMetadataEntry>
}

impl LeafRequestSize {
    pub(crate) fn new() -> LeafRequestSize {
        LeafRequestSize {
            base_range: RangeUsed::None,
            pixel_range: RangeUsed::None,
            height: 0.,
            metadata: vec![]
        }
    }

    pub(crate) fn base_range(&self) -> &RangeUsed<f64> { &self.base_range }
    pub(crate) fn pixel_range(&self) -> &RangeUsed<f64> { &self.pixel_range }
    pub(crate) fn height(&self) -> f64 { self.height }
    pub(crate) fn metadata(&self) -> &[AllotmentMetadataEntry] { &self.metadata }

    pub fn merge_height(&mut self, new_max: f64) { 
        self.height = self.height.max(new_max);
    }

    pub fn merge_base_range(&mut self, base_range: &RangeUsed<f64>) {
        self.base_range = self.base_range.merge(base_range);
    }

    pub fn merge_pixel_range(&mut self, new_range: &RangeUsed<f64>) {
        self.pixel_range = self.pixel_range.merge(new_range);
    }

    pub(crate) fn merge_metadata(&mut self, entry: AllotmentMetadataEntry) {
        self.metadata.push(entry);
    }
}
