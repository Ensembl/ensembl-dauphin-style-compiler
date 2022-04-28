use std::{sync::{Arc, Mutex}, collections::HashMap};

use crate::allotment::{style::allotmentname::AllotmentName, collision::bumpprocess::BumpPersistent};

pub(crate) struct TrainPersistent {
    bump: HashMap<AllotmentName,BumpPersistent>,
    bp_per_carriage: u64
}

impl TrainPersistent {
    pub(crate) fn new(bp_per_carriage: u64) -> TrainPersistent {
        TrainPersistent {
            bump: HashMap::new(),
            bp_per_carriage
        }
    }

    pub(crate) fn bump_mut(&mut self, name: &AllotmentName) -> &mut BumpPersistent {
        let bp_per_carriage = self.bp_per_carriage;
        self.bump.entry(name.clone()).or_insert_with(|| BumpPersistent::new(bp_per_carriage))
    }
}
