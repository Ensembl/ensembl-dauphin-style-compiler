use std::{sync::{Arc, Mutex}};

use crate::lock;

use super::{puzzle::{PuzzleSolution, PuzzleDependency}, graph::PuzzleGraph, answers::{Answers, AnswerIndex},};

pub(super) trait ErasedPiece {
    fn finish(&self, index: &AnswerIndex);
    fn apply_defaults(&self, solution: &mut PuzzleSolution);
}

pub trait PuzzleValue<T: 'static> {
    fn dependency(&self) -> PuzzleDependency;
    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>>;
    fn get(&self, solution: &PuzzleSolution) -> Arc<T> { self.try_get(solution).unwrap() }
}

pub trait ClonablePuzzleValue<T: 'static + Clone> : PuzzleValue<T> {
    fn try_get_clone(&self, solution: &PuzzleSolution) -> Option<T> {
        self.try_get(solution).map(|x| x.as_ref().clone())
    }

    fn get_clone(&self, solution: &PuzzleSolution) -> T {
        self.get(solution).as_ref().clone()
    }
}

pub struct PuzzlePiece<T> {
    graph: Arc<Mutex<PuzzleGraph>>,
    dependency: PuzzleDependency,
    answers: Answers<T>,
    default: Arc<Mutex<Option<T>>>
}

// Rust bug means dan't derive Clone on polymorphic types
impl<T> Clone for PuzzlePiece<T> {
    fn clone(&self) -> Self {
        PuzzlePiece {
            graph: self.graph.clone(),
            dependency: self.dependency.clone(),
            answers: self.answers.clone(),
            default: self.default.clone()
        }
    }
}

impl<T: 'static> PuzzlePiece<T> {
    pub(super) fn new(graph: &Arc<Mutex<PuzzleGraph>>, dependency: PuzzleDependency, default: Option<T>) -> PuzzlePiece<T> {
        PuzzlePiece {
            graph: graph.clone(),
            dependency,
            answers: Answers::new(),
            default: Arc::new(Mutex::new(default))
        }
    }

    pub(super) fn erased(&self) -> Box<dyn ErasedPiece> { Box::new(self.clone()) }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize { self.answers.len() }

    pub fn set_answer(&self, solution: &mut PuzzleSolution, value: T) {
        let index = self.answers.set(value);
        if !solution.set_answer_index(&self.dependency,&index) {
            /* double set: naughty user or default application */
            self.answers.finish(&index);
        }
        solution.just_answered().push(self.dependency.clone());
    }

    pub fn add_solver<F>(&self, dependencies: &[PuzzleDependency], callback: F) where F: Fn(&mut PuzzleSolution) -> Option<T> + 'static {
        let self2 = self.clone();
        /* Do all this in nesting callback to avoid polymorphism infecting solver */
        let solver = move |solution: &mut PuzzleSolution| {
            if let Some(answer) = callback(solution) {
                self2.set_answer(solution,answer);
            }
        };
        lock!(self.graph).add_solver(&self.dependency,dependencies,Arc::new(solver));
    }
}

impl<T: 'static> PuzzleValue<T> for PuzzlePiece<T> {
    fn dependency(&self) -> PuzzleDependency { self.dependency.clone() }

    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>> {
        let index = if let Some(x) = solution.get_answer_index(&self.dependency) { x } else { return None; };
        self.answers.get(&index)
    }
}

impl<T: 'static+ Clone> ClonablePuzzleValue<T> for PuzzlePiece<T> {}

impl<T: 'static> ErasedPiece for PuzzlePiece<T> {
    fn finish(&self, index: &AnswerIndex) {
        self.answers.finish(index);
    }

    fn apply_defaults(&self, solution: &mut PuzzleSolution) {
        if let Some(default) = lock!(self.default).take() {
            self.set_answer(solution,default);
        }
    }
}

#[cfg(test)]
mod text {
    use crate::puzzle::puzzle::Puzzle;

    use super::*;

    /* solver isbest tested at the puzzle level */

    #[test]
    fn piece_set() {
        let mut puzzle = Puzzle::new();
        let p1 = puzzle.new_piece(None);
        let mut s1 = PuzzleSolution::new(&puzzle);
        let mut s2 = PuzzleSolution::new(&puzzle);
        let mut s3 = PuzzleSolution::new(&puzzle);
        p1.set_answer(&mut s1,1);
        p1.set_answer(&mut s2,2);
        p1.set_answer(&mut s3,3);
        assert_eq!(1,p1.get_clone(&s1));
        assert_eq!(2,p1.get_clone(&s2));
        assert_eq!(3,p1.get_clone(&s3));
        drop(s1);
        assert_eq!(2,p1.get_clone(&s2));
        assert_eq!(3,p1.get_clone(&s3));
        assert_eq!(3,p1.len());
        drop(s3);
        assert_eq!(2,p1.get_clone(&s2));
        assert_eq!(2,p1.len());
        drop(s2);
        assert_eq!(0,p1.len());
    }

    #[test]
    fn piece_double_set() {
        let mut puzzle = Puzzle::new();
        let p1 = puzzle.new_piece(None);
        let mut s1 = PuzzleSolution::new(&puzzle);
        p1.set_answer(&mut s1,1);
        p1.set_answer(&mut s1,2);
        assert_eq!(1,p1.len());
        assert_eq!(1,p1.get_clone(&s1));
    }
}
