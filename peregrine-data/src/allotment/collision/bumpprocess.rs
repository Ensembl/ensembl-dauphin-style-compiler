use peregrine_toolkit::puzzle::PuzzleValueHolder;

use crate::allotment::util::rangeused::RangeUsed;

pub(crate) struct BumpRequest {
    range: PuzzleValueHolder<RangeUsed<u64>>,
    height: PuzzleValueHolder<i64>
}

struct BumpProcess {

}