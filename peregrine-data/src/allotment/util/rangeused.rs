use std::ops::{Add, Mul, Div};

fn partial_ord<T: PartialOrd>(a: T, b: T) -> (T,T) {
    if a < b { (a,b) } else { (b,a) }
}

fn partial_min<T: PartialOrd>(a: T, b: T) -> T { partial_ord(a,b).0 }
fn partial_max<T: PartialOrd>(a: T, b: T) -> T { partial_ord(a,b).1 }


#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub enum RangeUsed<T> {
    None,
    All,
    Part(T,T)
}

// XXX could this be merged with native ranges?
impl<T: Clone+PartialOrd+Add<Output=T>+Mul<Output=T>+Div<Output=T>> RangeUsed<T> {
    pub fn merge(&self, other: &RangeUsed<T>) -> RangeUsed<T> {
        match (self,other) {
            (RangeUsed::All,_) => RangeUsed::All,
            (_,RangeUsed::All) => RangeUsed::All,
            (RangeUsed::None,x) => x.clone(),
            (x,RangeUsed::None) => x.clone(),
            (RangeUsed::Part(a1,b1), RangeUsed::Part(a2,b2)) => {
                let (a1,b1) = partial_ord(a1,b1);
                let (a2,b2) = partial_ord(a2,b2);
                RangeUsed::Part(partial_min(a1,a2).clone(),partial_max(b1,b2).clone())
            }
        }
    }

    pub fn plus(&self, other: &RangeUsed<T>) -> RangeUsed<T> {
        match (self,other) {
            (RangeUsed::All,_) => RangeUsed::All,
            (_,RangeUsed::All) => RangeUsed::All,
            (RangeUsed::None,x) => x.clone(),
            (x,RangeUsed::None) => x.clone(),
            (RangeUsed::Part(a1,b1), RangeUsed::Part(a2,b2)) =>
                RangeUsed::Part(a1.clone()+a2.clone(),b1.clone()+b2.clone())
        }
    }

    pub fn plus_scalar(&self, delta: T) -> RangeUsed<T> {
        match self {
            RangeUsed::None => RangeUsed::None,
            RangeUsed::All => RangeUsed::All,
            RangeUsed::Part(a,b) => RangeUsed::Part(a.clone()+delta.clone(),b.clone()+delta)
        }
    }

    pub fn scale(&self, mul: T) -> RangeUsed<T> {
        match self {
            RangeUsed::Part(a,b) => RangeUsed::Part(a.clone()*mul.clone(),b.clone()*mul),
            x => x.clone()
        }
    }

    pub fn scale_recip(&self, div: T) -> RangeUsed<T> {
        match self {
            RangeUsed::Part(a,b) => RangeUsed::Part(a.clone()/div.clone(),b.clone()/div),
            x => x.clone()
        }
    }

    pub fn pixel_range(&self, pixel: &RangeUsed<T>, max_px_per_bp: T) -> RangeUsed<T> {
        pixel.plus(&self.scale(max_px_per_bp))
    }

    pub fn carriage_range(&self, pixel: &RangeUsed<T>, min_px_per_carriage: T, bp_per_carriage: T) -> RangeUsed<T> {
        //let max_carriage_for_pixel = pixel.scale_recip(min_px_per_carriage);
        let carriage_for_bp = self.scale_recip(bp_per_carriage);
        //max_carriage_for_pixel.plus(&carriage_for_bp)
        carriage_for_bp
    }

    pub fn into<F,U>(&self, cb: F) -> RangeUsed<U> where F: Fn(&T) -> U {
        match self {
            RangeUsed::None => RangeUsed::None,
            RangeUsed::All => RangeUsed::All,
            RangeUsed::Part(a,b) => RangeUsed::Part(cb(a),cb(b))
        }        
    }
}
