use std::{collections::HashSet, mem };
use super::{bumprequest::BumpRequestSet, algorithmbuilder::{BumpResponses, AlgorithmBuilder}};

pub(crate) trait GenericBumpingAlgorithm {
    fn add(&mut self, requests: &BumpRequestSet) -> bool;
    fn build(&self) -> BumpResponses;
}

pub(crate) struct BumpPersistent {
    wanted: HashSet<usize>,
    use_wall: bool,
    algorithm: Option<Box<dyn GenericBumpingAlgorithm>>,
    bumper_number: u64
}

impl BumpPersistent {
    pub(crate) fn new(use_wall: bool) -> BumpPersistent {
        BumpPersistent {
            wanted: HashSet::new(),
            algorithm: None,
            use_wall,
            bumper_number: 0
        }
    }

    fn try_only_new(&mut self, new: &[BumpRequestSet]) -> bool {
        let algorithm = self.algorithm.as_mut().unwrap();
        for new in new {
            if !algorithm.add(new) { return false; }
        }
        true
    }

    pub(crate) fn make(&mut self, input: &[BumpRequestSet]) -> (BumpResponses,u64) {
        let new_all_wanted = input.iter().map(|x| x.index()).collect::<HashSet<_>>();
        /* Perfect match? */
        if let Some(bumper) = &self.algorithm {
            if new_all_wanted == self.wanted {
                return (bumper.build(),self.bumper_number);
            }
        }
        let old_wanted = mem::replace(&mut self.wanted, new_all_wanted);
        if self.algorithm.is_some() {
            /* Try incremental */
            let new_new_wanted = self.wanted.difference(&old_wanted).cloned().collect::<Vec<_>>();
            let newly_wanted = input.iter().filter(|x| new_new_wanted.contains(&x.index())).cloned().collect::<Vec<_>>();
            if self.try_only_new(&newly_wanted) {
                return (self.algorithm.as_ref().unwrap().build(),self.bumper_number);
            }
        }
        /* Rebuild completely */
        let mut builder = AlgorithmBuilder::new();
        for set in input {
            builder.add(&set);
        }
        let bumper = if self.use_wall {
            Box::new(builder.make_standard(true)) as Box<dyn GenericBumpingAlgorithm>
        } else {
            Box::new(builder.make_standard(false)) as Box<dyn GenericBumpingAlgorithm>
        };
        self.algorithm = Some(bumper);
        (self.algorithm.as_ref().unwrap().build(),self.bumper_number)
    }
}
