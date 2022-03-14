use peregrine_toolkit::puzzle::PuzzleValueHolder;

#[derive(PartialEq,Clone,Debug)]
pub struct PlayingField {
    height: f64,
    squeeze: (f64,f64),
}

impl PlayingField {
    pub fn empty() -> PlayingField {
        PlayingField {
            height: 0.,
            squeeze: (0.,0.)
        }
    }

    pub fn new_height(height: f64) -> PlayingField {
        PlayingField {
            height,
            squeeze: (0.,0.)
        }
    }

    pub fn new_squeeze(left: f64, right: f64) -> PlayingField {
        PlayingField {
            height: 0.,
            squeeze: (left,right)
        }
    }

    pub fn height(&self) -> f64 { self.height }
    pub fn squeeze(&self) -> (f64,f64) { self.squeeze }

    pub fn union(&mut self, other: &PlayingField) {
        self.height = self.height.max(other.height);
        self.squeeze.0 = self.squeeze.0.max(other.squeeze.0);
        self.squeeze.1 = self.squeeze.1.max(other.squeeze.1);
    }
}
