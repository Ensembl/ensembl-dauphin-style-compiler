use std::cmp::{ Ordering, PartialOrd };

#[derive(PartialEq,Clone,Copy)]
pub enum Precision {
    Float(i32,i32),
    Int(i32)
}

fn strictly_less(ap: i32, ar: i32, bp: i32, br: i32) -> bool {
    (ap<bp || ar<br) && ap<=bp && ar<=br
}

impl PartialOrd for Precision {
    fn partial_cmp(&self, other: &Precision) -> Option<Ordering> {
        match (*self,*other) {
            (Precision::Float(ap,ar),Precision::Float(bp,br)) => {
                if strictly_less(ap,ar,bp,br) { return Some(Ordering::Less); }
                if strictly_less(bp,br,ap,ar) { return Some(Ordering::Greater); }
                if other == self { return Some(Ordering::Equal); }
                return None;
            },
            (Precision::Float(_,_),Precision::Int(_)) => { return Some(Ordering::Greater); },
            (Precision::Int(_),Precision::Float(_,_)) => { return Some(Ordering::Less); },
            (Precision::Int(a),Precision::Int(b)) => { return Some(a.cmp(&b)) }

        }
    }
}