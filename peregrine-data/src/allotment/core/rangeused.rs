#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub enum RangeUsed {
    None,
    All,
    Part(f64,f64)
}

// XXX could this be merged with native ranges?
impl RangeUsed {
    pub fn merge(&self, other: &RangeUsed) -> RangeUsed {
        match (self,other) {
            (RangeUsed::All,_) => RangeUsed::All,
            (_,RangeUsed::All) => RangeUsed::All,
            (RangeUsed::None,x) => x.clone(),
            (x,RangeUsed::None) => x.clone(),
            (RangeUsed::Part(a1,b1), RangeUsed::Part(a2,b2)) => {
                let (a1,b1) = if a1<b1 { (a1,b1) } else { (b1,a1) };
                let (a2,b2) = if a2<b2 { (a2,b2) } else { (b2,a2) };
                RangeUsed::Part(a1.min(*a2),b1.max(*b2))
            }
        }
    }

    pub fn plus(&self, other: &RangeUsed) -> RangeUsed {
        match (self,other) {
            (RangeUsed::All,_) => RangeUsed::All,
            (_,RangeUsed::All) => RangeUsed::All,
            (RangeUsed::None,x) => x.clone(),
            (x,RangeUsed::None) => x.clone(),
            (RangeUsed::Part(a1,b1), RangeUsed::Part(a2,b2)) => RangeUsed::Part(*a1+*a2,*b1+*b2)
        }
    }

    pub fn plus_scalar(&self, delta: f64) -> RangeUsed {
        match self {
            RangeUsed::None => RangeUsed::None,
            RangeUsed::All => RangeUsed::All,
            RangeUsed::Part(a,b) => RangeUsed::Part(a+delta,b+delta)
        }
    }

    pub fn scale(&self, mul: f64) -> RangeUsed {
        match self {
            RangeUsed::Part(a,b) => RangeUsed::Part(a*mul,b*mul),
            x => x.clone()
        }
    }

    pub fn pixel_range(&self, pixel: &RangeUsed, max_px_per_bp: f64) -> RangeUsed {
        pixel.plus(&self.scale(max_px_per_bp))
    }
}
