use std::{sync::{Arc, Mutex}, collections::HashMap};

use peregrine_toolkit::{lock, puzzle::{Answer, StaticValue, StaticAnswer}};

use crate::allotment::{util::rangeused::RangeUsed, style::allotmentname::AllotmentName};

use super::concretebump::{ConcreteRequests, ConcreteBump, ConcreteResults};

use lazy_static::lazy_static;
use identitynumber::identitynumber;

#[derive(Clone)]
pub(crate) struct BumpRequest {
    name: AllotmentName,
    range: StaticValue<RangeUsed<f64>>,
    height: StaticValue<f64>
}

impl BumpRequest {
    fn add_to_concrete_requests(&self, answer_index: &mut StaticAnswer, requests: &mut ConcreteRequests) {
        match self.range.call(answer_index) {
            RangeUsed::None =>
                {},
            RangeUsed::All =>
                requests.add_infinite(&self.name,self.height.call(answer_index).ceil() as i64),
            RangeUsed::Part(start,end) =>
                requests.add_finite(&self.name,start.floor().max(0.) as u64,end.ceil().max(0.) as u64,self.height.call(answer_index).ceil() as i64),
        }
    }
}

pub(crate) struct BumpRequests {
    requests: Vec<BumpRequest>,
    index: usize
}

impl BumpRequests {
    pub(crate) fn new(index: usize) -> BumpRequests {
        let mut out = BumpRequests {
            requests: vec![],
            index
        };
        out
    }

    pub(crate) fn add(&mut self, name: &AllotmentName, range: &StaticValue<RangeUsed<f64>>, height: &StaticValue<f64>) {
        self.requests.push(BumpRequest {
            name: name.clone(),
            range: range.clone(),
            height: height.clone()
        })
    }

    fn add_concrete(&self, answer_index: &mut StaticAnswer, concrete: &mut ConcreteBump) -> bool {
        let mut requests = concrete.new_requests(self.index);
        for request in &*self.requests {
            request.add_to_concrete_requests(answer_index,&mut requests);
        }
        concrete.add(requests)
    }

}

/*
pub(crate) struct BumpResponses {
    piece: PuzzlePiece<ConcreteResults>,
    process: BumpProcess,
    index: usize
}

impl BumpResponses {
    
}

impl PuzzleValue<ConcreteResults> for BumpResponses {
    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<ConcreteResults>> {
        Some(self.process.get_result(solution,self.index))
    }

    fn known_constant_value(&self) -> Option<Arc<ConcreteResults>> { None }
    fn dependency(&self) -> PuzzleDependency { self.piece.dependency() }
}
*/

/* Each solution can have a different ConcreteBump and (more often) a different maximum height.
 * The call to ConcreteBump which alters the height and, potentially, causes it to be useless and
 * need to be regenerated is add(). This means that add_carriage() would be solution-specific
 * even if none of its arguments were so, as the combination of such adds *is* solution specific and
 * this affects the answer to each. However, in general, of course, the components *may* be solution-specific
 * as well, as they are PuzzleValueHolders. So the boxes need to hang on to BumpRequests and addthem each time
 * on a per-solution basis. BumpProcess provides state in the form of (ConcreteBump-serial,max_height) and 
 * causes new DrawingCarriages (and so new solutions) when this changes. 
 * 
 * We also need to know when we can dispose of a ConcreteBump (and any entry in our lookup). To achieve this,
 * we use the PuzzleSolution's drop hook (and is, infact, the reason they were created). Internally we use a
 * BumpSolution to represent the (ConcreteBump,height) pair and it's this object which contains the id for the
 * TrainState.
 * 
 * Note: bumping means that we *CANNOT* say that the height of a track is determined by the largest of the
 * heights of the tracks in each Carriage. It could be that a greater height is required to bump all the
 * carriages correctly. 
 */

identitynumber!(IDS);

struct BumpSolution {
    concrete: Arc<Mutex<ConcreteBump>>,
    height: u64,
    id: u64
}

impl BumpSolution {
    fn add(&self, answer_index: &mut StaticAnswer, requests: &BumpRequests) -> bool {
        requests.add_concrete(answer_index,&mut *lock!(self.concrete))
    }
}

/*
#[derive(Clone)]
pub(crate) struct BumpProcess {
    bp_per_carriage: u64,
    current_concrete: Arc<Mutex<Option<ConcreteBump>>>,
    solutions: Arc<Mutex<HashMap<PuzzleSolution,BumpSolution>>>
}

impl BumpProcess {
    pub(crate) fn new(bp_per_carriage: u64) -> BumpProcess {
        BumpProcess {
            bp_per_carriage,
            current_concrete: Arc::new(Mutex::new(None)),
            solutions: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    fn get_result(&self, solution: &PuzzleSolution, index: usize) -> Arc<ConcreteResults> {
        todo!()
    }

    pub(crate) fn add_carriage(&mut self, solution: &PuzzleSolution, requests: &BumpRequests) -> BumpResponses {
        todo!()
    }
}
*/