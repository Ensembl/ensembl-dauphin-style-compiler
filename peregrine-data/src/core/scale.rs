use serde::Serializer;
use std::fmt::{ self, Display, Formatter };
use serde_cbor::Value as CborValue;

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

    pub fn prev_scale(&self) -> Option<Scale> {
        if self.0 > 0 {
            Some(Scale(self.0-1))
        } else {
            None
        }
    }

    pub fn next_scale(&self) -> Scale {
        Scale(self.0+1)
    }

    pub fn get_index(&self) -> u64 {
        self.0
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
