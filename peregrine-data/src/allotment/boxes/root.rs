use peregrine_toolkit::puzzle::{ConstantPuzzlePiece, PuzzleValueHolder};

use super::boxtraits::Stackable;

#[derive(Clone)]
pub struct Root {

}

impl Root {
    pub fn new() -> Root { Root {} }

    pub fn add_child(&self, child: &dyn Stackable) {
        child.set_indent(&PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)));
        child.set_top(&PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)));
    }    
}
