use std::{sync::{Arc}, collections::HashSet};
use peregrine_toolkit::log;

use crate::allotment::collision::collisionalgorithm2::bump;

use super::{concretebump::{ConcreteRequests, ConcreteResults, ConcreteBump}, collisionalgorithm2::{BumpRequestSet, BumpResponses}};

pub(crate) struct BumpPersistent {
    wanted: HashSet<u64>,
    bp_per_carriage: u64,
    responses: Option<BumpResponses>,
    bumper_number: u64
}

impl BumpPersistent {
    pub(crate) fn new(bp_per_carriage: u64) -> BumpPersistent {
        BumpPersistent {
            wanted: HashSet::new(),
            bp_per_carriage,
            responses: None,
            bumper_number: 0
        }
    }

    pub(crate) fn make(&mut self, input: &[Arc<BumpRequestSet>]) -> (BumpResponses,u64) {
        if let Some(bumper) = &self.responses {
            let new_wanted = input.iter().map(|x| x.identity()).collect::<HashSet<_>>();
            if new_wanted == self.wanted {
//                return (bumper.clone(),self.bumper_number);
            }
        }
        self.bumper_number += 1;
        let inputs = input.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
        self.responses = Some(bump(&inputs));
        (self.responses.as_ref().unwrap().clone(),self.bumper_number)
    }
}
