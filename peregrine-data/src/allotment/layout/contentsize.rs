use peregrine_toolkit::puzzle::{StaticValue, derived};
use crate::{allotment::{core::{allotmentname::AllotmentName, rangeused::RangeUsed}}, shape::metadata::AllotmentMetadataEntry};

pub(crate) struct ContentSize {
    pub(crate) height: StaticValue<f64>,
    pub(crate) range: RangeUsed<f64>,
    pub(crate) metadata: Vec<AllotmentMetadataEntry>
}

impl ContentSize {
    pub(crate) fn to_value(&self, name: &AllotmentName) -> StaticValue<(AllotmentName,f64,RangeUsed<f64>)> {
        let name = name.clone();
        let range = self.range.clone();
        derived(self.height.clone(),move |h| {
            (name.clone(),h,range.clone())
        })
    }
}
