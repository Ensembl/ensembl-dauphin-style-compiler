use std::{sync::{Arc, Mutex}, mem};

use crate::lock;

use super::{puzzle::{PuzzleSolution, PuzzleDependency}, graph::PuzzleGraph, answers::{Answers, AnswerIndex},};

pub(super) trait ErasedPiece {
    fn puzzle_ready(&mut self);
    fn finish(&self, index: &AnswerIndex);
    fn apply_defaults(&mut self, solution: &mut PuzzleSolution);
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

pub struct PuzzleValueHolder<T: 'static>(Arc<dyn PuzzleValue<T>>);

impl<T: 'static> PuzzleValueHolder<T> {
    pub fn new<F>(value: F) -> PuzzleValueHolder<T> where F: PuzzleValue<T> + 'static {
        PuzzleValueHolder(Arc::new(value))
    }
}

impl<T: 'static> PuzzleValue<T> for PuzzleValueHolder<T> {
    fn dependency(&self) -> PuzzleDependency { self.0.dependency() }
    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>> { self.0.try_get(solution) }
}

impl<T: 'static+ Clone> ClonablePuzzleValue<T> for PuzzleValueHolder<T> {}

impl<T: 'static> Clone for PuzzleValueHolder<T> {
    fn clone(&self) -> Self { Self(self.0.clone()) }
}

pub struct PuzzlePiece<T> {
    graph: Arc<Mutex<PuzzleGraph>>,
    dependency: PuzzleDependency,
    answers: Answers<T>,
    default: Arc<Mutex<Option<T>>>,
    readies: Arc<Mutex<Vec<Box<dyn FnOnce(&mut PuzzlePiece<T>) + 'static>>>>
}

impl<T> Clone for PuzzlePiece<T> {
    fn clone(&self) -> Self {
        PuzzlePiece {
            graph: self.graph.clone(),
            dependency: self.dependency.clone(),
            answers: self.answers.clone(),
            default: self.default.clone(),
            readies: self.readies.clone()
        }
    }
}

impl<T: 'static> PuzzlePiece<T> {
    pub(super) fn new(graph: &Arc<Mutex<PuzzleGraph>>, dependency: PuzzleDependency, default: Option<T>) -> PuzzlePiece<T> {
        PuzzlePiece {
            graph: graph.clone(),
            dependency,
            answers: Answers::new(),
            default: Arc::new(Mutex::new(default)),
            readies: Arc::new(Mutex::new(vec![]))
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
        *lock!(self.default) = None; // rely on solver
        let self2 = self.clone();
        /* Do all this in nesting callback to avoid polymorphism infecting solver */
        let solver = move |solution: &mut PuzzleSolution| {
            if let Some(answer) = callback(solution) {
                self2.set_answer(solution,answer);
            }
        };
        lock!(self.graph).add_solver(&self.dependency,dependencies,Arc::new(solver));
    }

    pub fn add_ready<F>(&self, cb: F) where F: FnOnce(&mut PuzzlePiece<T>) + 'static {
        lock!(self.readies).push(Box::new(cb))
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
    fn puzzle_ready(&mut self) {
        let readies = mem::replace(lock!(self.readies).as_mut(),vec![]);
        for ready in readies {
            ready(self);
        }
    }

    fn finish(&self, index: &AnswerIndex) {
        self.answers.finish(index);
    }

    fn apply_defaults(&mut self, solution: &mut PuzzleSolution) {
        if let Some(default) = lock!(self.default).take() {
            self.set_answer(solution,default);
        }
    }
}

#[cfg(test)]
mod text {
    use crate::puzzle::puzzle::{Puzzle, PuzzleBuilder};

    use super::*;

    /* solver isbest tested at the puzzle level */
    
    #[test]
    fn piece_set() {
        let mut builder = PuzzleBuilder::new();
        let p1 = builder.new_piece(None);
        let puzzle = Puzzle::new(builder);
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
        let mut builder = PuzzleBuilder::new();
        let p1 = builder.new_piece(None);
        let puzzle = Puzzle::new(builder);
        let mut s1 = PuzzleSolution::new(&puzzle);
        p1.set_answer(&mut s1,1);
        p1.set_answer(&mut s1,2);
        assert_eq!(1,p1.len());
        assert_eq!(1,p1.get_clone(&s1));
    }

    #[test]
    fn piece_ready() {
        let mut builder = PuzzleBuilder::new();
        let flag = Arc::new(Mutex::new(false));
        let p1 : PuzzlePiece<()> = builder.new_piece(None);
        let flag2 = flag.clone();
        p1.add_ready(move |p| {
            *lock!(flag2) = true;
        });
        assert_eq!(false,*lock!(flag));
        let _puzzle = Puzzle::new(builder);
        assert_eq!(true,*lock!(flag));
    }
}
