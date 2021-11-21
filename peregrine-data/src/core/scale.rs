use std::fmt::{ self, Display, Formatter };
use serde_cbor::Value as CborValue;

const MILESTONE_GAP : u64 = 4;

#[derive(Clone,Debug,Eq,PartialEq,Hash)]
pub struct Scale(u64);

impl Scale {
    pub fn new(scale: u64) -> Scale {
        Scale(scale)
    }

    pub fn encode(&self) -> CborValue {
        CborValue::Integer(self.0 as i128)
    }

    pub fn new_bp_per_screen(bp_per_screen: f64) -> Scale {
        Scale(bp_per_screen.log2().floor() as u64)
    }

    pub fn bp_per_screen_range(&self) -> (u64,u64) {
        let bp_in_carriage = self.bp_in_carriage();
        (bp_in_carriage,bp_in_carriage*2-1)
    }

    pub fn delta_scale(&self, amt: i64) -> Option<Scale> {
        let new_scale = (self.0 as i64) + amt;
        if new_scale >= 0 {
            Some(Scale(new_scale as u64))
        } else {
            None
        }
    }

    pub fn get_index(&self) -> u64 {
        self.0
    }

    pub fn is_milestone(&self) -> bool {
        self.0 % MILESTONE_GAP == 0
    }

    pub fn to_milestone(&self) -> Scale {
        let new_scale = ((self.0+MILESTONE_GAP-1)/MILESTONE_GAP)*MILESTONE_GAP; // round up
        Scale(new_scale)
    }

    pub fn bp_in_carriage(&self) -> u64 {
        1 << self.0
    }

    pub fn convert_index(&self, old_scale: &Scale, old_index: u64) -> u64 {
        let log_bp_factor = (self.0 as i64) - (old_scale.0 as i64);
        if log_bp_factor >= 0 {
            /* zooming out so we are entirely contained => easy */
            old_index / (1<<log_bp_factor)
        } else {
            /* zooming in, choose centre */
            let left = old_index * (1<<(-log_bp_factor));
            left + (1<<(-log_bp_factor-1))
        }
    }

    pub fn carriage(&self, position: f64) -> u64 {
        (position / (self.bp_in_carriage() as f64)).floor() as u64
    }
}

impl Display for Scale {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}
