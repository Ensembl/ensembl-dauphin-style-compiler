use std::sync::{Arc, Mutex};

use crate::lock;

use super::{piece::{PuzzleValue, ClonablePuzzleValue}, puzzle::{PuzzleDependency, PuzzleSolution, DelaySlot}, PuzzleBuilder, PuzzleValueHolder};

#[derive(Clone)]
pub struct DerivedPuzzlePiece<T,U> {
    value: Arc<dyn PuzzleValue<T>>,
    callback: Arc<Box<dyn Fn(&T) -> U>>
}

impl<T: 'static,U> DerivedPuzzlePiece<T,U> {
    pub fn new<F,V: 'static>(value: V, cb: F) -> DerivedPuzzlePiece<T,U> where F: Fn(&T) -> U + 'static, V: PuzzleValue<T> {
        DerivedPuzzlePiece {
            value: Arc::new(value),
            callback: Arc::new(Box::new(cb))
        }
    }
}

impl<T: 'static,U: 'static> PuzzleValue<U> for DerivedPuzzlePiece<T,U> {
    fn dependency(&self) -> PuzzleDependency {
        self.value.dependency()
    }

    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<U>> {
       self.value.try_get(solution).map(|t| Arc::new((self.callback)(&t)))
    }
}

impl<T: 'static,U: 'static + Clone> ClonablePuzzleValue<U> for DerivedPuzzlePiece<T,U> {}

pub struct ConstantPuzzlePiece<T>(Arc<T>);

impl<T> ConstantPuzzlePiece<T> {
    pub fn new(value: T) -> ConstantPuzzlePiece<T> {
        ConstantPuzzlePiece(Arc::new(value))
    }
}

impl<T: Clone> Clone for ConstantPuzzlePiece<T> {
    fn clone(&self) -> Self { ConstantPuzzlePiece(self.0.clone()) }
}

impl<T: 'static> PuzzleValue<T> for ConstantPuzzlePiece<T> {
    fn dependency(&self) -> PuzzleDependency { PuzzleDependency::constant() }

    fn try_get(&self, _solution: &PuzzleSolution) -> Option<Arc<T>> {
        Some(self.0.clone())
    }
}

impl<T: Clone + 'static> ClonablePuzzleValue<T> for ConstantPuzzlePiece<T> {}

pub struct DelayedPuzzleValue<T: 'static>(DelaySlot,Arc<Mutex<Option<PuzzleValueHolder<T>>>>);

impl<T> DelayedPuzzleValue<T> {
    pub fn new(builder: &PuzzleBuilder) -> DelayedPuzzleValue<T> {
        DelayedPuzzleValue(builder.allocate_delayed(),Arc::new(Mutex::new(None)))
    }

    pub fn set(&self, builder: &PuzzleBuilder, target: PuzzleValueHolder<T>) {
        builder.set_delayed(&self.0,target.dependency());
        *lock!(self.1) = Some(target);
    }
}

impl<T: Clone> Clone for DelayedPuzzleValue<T> {
    fn clone(&self) -> Self { DelayedPuzzleValue(self.0.clone(),self.1.clone()) }
}

impl<T: 'static> PuzzleValue<T> for DelayedPuzzleValue<T> {
    fn dependency(&self) -> PuzzleDependency { PuzzleDependency::delayed(&self.0) }

    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>> {
        lock!(self.1).as_ref().and_then(|x| x.try_get(solution))
    }
}

impl<T: Clone + 'static> ClonablePuzzleValue<T> for DelayedPuzzleValue<T> {}

pub struct DelayedConstant<T>(Arc<Mutex<Option<Arc<T>>>>);

impl<T> DelayedConstant<T> {
    pub fn new() -> DelayedConstant<T> {
        DelayedConstant(Arc::new(Mutex::new(None)))
    }

    pub fn set(&self, value: T) {
        *lock!(self.0) = Some(Arc::new(value));
    }
}

impl<T:Clone> Clone for DelayedConstant<T> {
    fn clone(&self) -> Self { DelayedConstant(self.0.clone()) } 
}

impl<T: 'static> PuzzleValue<T> for DelayedConstant<T> {
    fn dependency(&self) -> PuzzleDependency { PuzzleDependency::constant() }

    fn try_get(&self, _solution: &PuzzleSolution) -> Option<Arc<T>> {
        lock!(self.0).as_ref().cloned()
    }
}

impl<T: Clone + 'static> ClonablePuzzleValue<T> for DelayedConstant<T> {}

#[cfg(test)]
mod test {
    use crate::puzzle::PuzzleBuilder;
    use crate::puzzle::puzzle::Puzzle;
    use crate::puzzle::piece::{PuzzleValue, ClonablePuzzleValue};

    use super::*;

    #[test]
    fn derived() {
        let builder = PuzzleBuilder::new();
        let p1 = builder.new_piece();
        let p2 = DerivedPuzzlePiece::new(p1.clone(),|x| *x*5);
        let p3 = builder.new_piece();
        let p2b = p2.clone();
        p3.add_solver(&[p2.dependency()], move |solution| {
            Some(p2b.get_clone(solution) + 2)
        });
        let puzzle = Puzzle::new(builder);
        let mut s1 = PuzzleSolution::new(&puzzle);
        p1.set_answer(&mut s1,2);
        s1.solve();
        assert_eq!(Some(2),p1.try_get_clone(&s1));
        assert_eq!(Some(10),p2.try_get_clone(&s1));
        assert_eq!(Some(12),p3.try_get_clone(&s1));
    }

    #[test]
    fn constant() {
        let builder = PuzzleBuilder::new();
        let p1 = ConstantPuzzlePiece::new(3);
        let p2 = DerivedPuzzlePiece::new(p1.clone(),|x| *x*5);
        let p3 = builder.new_piece();
        let p2b = p2.clone();
        p3.add_solver(&[p2.dependency()], move |solution| {
            Some(p2b.get_clone(solution) + 2)
        });
        let puzzle = Puzzle::new(builder);
        let mut s1 = PuzzleSolution::new(&puzzle);
        s1.solve();
        assert_eq!(Some(3),p1.try_get_clone(&s1));
        assert_eq!(Some(15),p2.try_get_clone(&s1));
        assert_eq!(Some(17),p3.try_get_clone(&s1));
    }
}
