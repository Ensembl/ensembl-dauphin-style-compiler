use std::{ops::{Add, Mul, Div, Sub}, cmp::Ordering, collections::btree_map::Range};

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

impl<T: PartialEq> PartialEq for RangeUsed<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Part(l0, l1), Self::Part(r0, r1)) => l0 == r0 && l1 == r1,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<T: PartialEq> Eq for RangeUsed<T> {}

impl<T: Clone+Sub<Output=T>+PartialOrd> PartialOrd<RangeUsed<T>> for RangeUsed<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match (self,other) {
            (RangeUsed::None, RangeUsed::None) => Ordering::Equal,
            (RangeUsed::None, RangeUsed::All) => Ordering::Less,
            (RangeUsed::None, RangeUsed::Part(_, _)) => Ordering::Less,
            (RangeUsed::All, RangeUsed::None) => Ordering::Greater,
            (RangeUsed::All, RangeUsed::All) => Ordering::Equal,
            (RangeUsed::All, RangeUsed::Part(_, _)) => Ordering::Greater,
            (RangeUsed::Part(_, _), RangeUsed::None) => Ordering::Greater,
            (RangeUsed::Part(_, _), RangeUsed::All) => Ordering::Less,
            (RangeUsed::Part(a,b), RangeUsed::Part(c,d)) => diff(b,a).partial_cmp(&diff(d,c)).unwrap()
        })
    }
}

fn diff<T>(a: &T, b: &T) -> T where T: Clone+Sub<Output=T> {
    a.clone()-b.clone()
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

    pub fn remove(&self, to_remove: &RangeUsed<T>) -> Vec<RangeUsed<T>> {
        match (self,to_remove) {
            (_,RangeUsed::None) => vec![],
            (_,RangeUsed::All) => vec![],
            (RangeUsed::All,_) => vec![RangeUsed::All],
            (RangeUsed::None,_) => vec![],
            (RangeUsed::Part(a,b),RangeUsed::Part(x,y)) => {
                let mut out = vec![];
                /* There's some remnant to the left if we cut at x after a but before b */
                if x > a && x <= b {
                    out.push(RangeUsed::Part(a.clone(),x.clone()));
                }
                /* There's some remnant to the right if we cut at y after a but before b */
                if y > a && y < b {
                    out.push(RangeUsed::Part(y.clone(),b.clone()));
                }
                out
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
