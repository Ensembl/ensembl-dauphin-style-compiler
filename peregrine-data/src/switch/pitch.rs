#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq)]
pub struct Pitch {
    y_height: i64
}

impl Pitch {
    pub(crate) fn new() -> Pitch {
        Pitch {
            y_height: 0
        }
    }

    pub(crate) fn set_limit(&mut self, y: i64) {
        self.y_height = self.y_height.max(y);
    }

    pub(crate) fn merge(&mut self, other: &Pitch) {
        self.set_limit(other.y_height);
    }

    pub fn height(&self) -> i64 { self.y_height }
}
