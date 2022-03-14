use std::sync::Arc;

use super::{piece::{PuzzleValue, ClonablePuzzleValue}, puzzle::{PuzzleDependency, PuzzleSolution}};

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
    fn dependency(&self) -> PuzzleDependency { PuzzleDependency::none() }

    fn try_get(&self, _solution: &PuzzleSolution) -> Option<Arc<T>> {
        Some(self.0.clone())
    }
}

impl<T: Clone + 'static> ClonablePuzzleValue<T> for ConstantPuzzlePiece<T> {}

#[cfg(test)]
mod test {
    use crate::puzzle::PuzzleBuilder;
    use crate::puzzle::puzzle::Puzzle;
    use crate::puzzle::piece::{PuzzleValue, ClonablePuzzleValue};

    use super::*;

    #[test]
    fn derived() {
        let mut builder = PuzzleBuilder::new();
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
        let mut builder = PuzzleBuilder::new();
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
