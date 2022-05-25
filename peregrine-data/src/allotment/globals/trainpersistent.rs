use std::{collections::HashMap};

use crate::allotment::{collision::bumpprocess::BumpPersistent, core::allotmentname::AllotmentName};

pub(crate) struct TrainPersistent {
    bump: HashMap<AllotmentName,BumpPersistent>
}

impl TrainPersistent {
    pub(crate) fn new() -> TrainPersistent {
        TrainPersistent {
            bump: HashMap::new()
        }
    }

    pub(crate) fn bump_mut(&mut self, name: &AllotmentName) -> &mut BumpPersistent {
        self.bump.entry(name.clone()).or_insert_with(|| BumpPersistent::new())
    }
}
