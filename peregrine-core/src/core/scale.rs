use std::fmt;

#[derive(Clone,Debug,Eq,PartialEq,Hash)]
pub struct Scale(u64);

impl Scale {
    pub fn new(scale: u64) -> Scale {
        Scale(scale)
    }

    pub fn prev_scale(&self) -> Scale {
        Scale(self.0-1)
    }

    pub fn next_scale(&self) -> Scale {
        Scale(self.0+1)
    }

    pub fn get_index(&self) -> u64 {
        self.0
    }

    pub fn bp_in_scale(&self) -> u64 {
        1 << self.0
    }
}