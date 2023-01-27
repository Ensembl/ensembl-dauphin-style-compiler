/* Too avoid too much rearranging when we tidy up unused BumpRequestSets, we always
 * add their contents in the same order and the caller also add the BumpRequestSets
 * themselves in the same order. To allow this, an ordering is provided on BumpRequestSets.
 * 
 * This should mean that items tend to "stay where they are" except when it is
 * advanatgeous to move them (down) due to now dead big teetering off-screen stacks.
 */

use std::sync::Arc;
use crate::allotment::core::{allotmentname::AllotmentName, rangeused::RangeUsed};
use peregrine_toolkit::{identitynumber, hashable, orderable};

const PIXEL_PRECISION : f64 = 1000000.;

#[derive(Clone)]
pub struct BumpRequest {
    pub(super) name: AllotmentName,
    pub(super) range: RangeUsed<f64>,
    pub(super) height: f64
}

impl BumpRequest {
    pub fn new(name: &AllotmentName, range: &RangeUsed<f64>, height: f64) -> BumpRequest {
        BumpRequest {
            name: name.clone(),
            range: range.clone(),
            height
        }
    }

    fn add_delta(&mut self, delta: f64) -> bool {
        match &mut self.range {
            RangeUsed::Part(a,b) => {
                *a += delta*PIXEL_PRECISION;
                *b += delta*PIXEL_PRECISION;
                true
            },
            RangeUsed::All => { true },
            RangeUsed::None => { false }
        }
    }
}

identitynumber!(IDS);
hashable!(BumpRequestSet,identity);
orderable!(BumpRequestSet,identity);

pub struct BumpRequestSetBuilder {
    members: Vec<BumpRequest>,
    index: usize
}

impl BumpRequestSetBuilder {
    pub(crate) fn new(index: usize) -> BumpRequestSetBuilder {
        BumpRequestSetBuilder { members: vec![], index }
    }

    pub(crate) fn add(&mut self, mut req: BumpRequest) {
        if req.add_delta(self.index as f64) {
            self.members.push(req);
        }
    }
}

#[derive(Clone)]
pub struct BumpRequestSet {
    pub(super) values: Arc<Vec<BumpRequest>>,
    pub(super) index: usize,
    identity: u64
}

impl BumpRequestSet {
    pub(crate) fn new(mut builder: BumpRequestSetBuilder) -> BumpRequestSet {
        builder.members.sort_by(|b,a| a.range.partial_cmp(&b.range).unwrap());
        BumpRequestSet {
            values: Arc::new(builder.members),
            identity: IDS.next(),
            index: builder.index
        }
    }

    pub(crate) fn index(&self) -> usize { self.index }
}
