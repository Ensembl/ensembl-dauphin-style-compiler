use peregrine_toolkit::puzzle::{StaticValue, derived};
use crate::{allotment::{core::allotmentname::AllotmentName, util::rangeused::RangeUsed}, shape::metadata::AllotmentMetadataEntry};

pub(crate) struct ContentSize {
    pub(crate) name: AllotmentName,
    pub(crate) height: StaticValue<f64>,
    pub(crate) range: RangeUsed<f64>,
    pub(crate) metadata: Vec<AllotmentMetadataEntry>
}

impl ContentSize {
    pub(crate) fn to_value(&self) -> StaticValue<(AllotmentName,f64,RangeUsed<f64>)> {
        let name = self.name.clone();
        let range = self.range.clone();
        derived(self.height.clone(),move |h| {
            (name.clone(),h,range.clone())
        })
    }
}
