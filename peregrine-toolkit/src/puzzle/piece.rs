use std::{sync::{Arc, Mutex}, mem};

use crate::lock;

use super::{puzzle::{PuzzleSolution, PuzzleDependency}, graph::PuzzleGraph, answers::{Answers, AnswerIndex},};

pub(super) trait ErasedPiece {
    fn puzzle_ready(&mut self);
    fn finish(&self, index: &AnswerIndex);
    fn apply_defaults(&mut self, solution: &mut PuzzleSolution, post: bool);
    fn is_solved(&self, solution: &PuzzleSolution) -> bool;
    fn erased_dependency(&self) -> PuzzleDependency;
}

pub trait PuzzleValue<T: 'static> {
    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>>;
    fn get(&self, solution: &PuzzleSolution) -> Arc<T> { self.try_get(solution).unwrap() }
    fn dependency(&self) -> PuzzleDependency;
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
    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>> { self.0.try_get(solution) }

    fn dependency(&self) -> PuzzleDependency { self.0.dependency() }
}

impl<T: 'static+ Clone> ClonablePuzzleValue<T> for PuzzleValueHolder<T> {}

impl<T: 'static> Clone for PuzzleValueHolder<T> {
    fn clone(&self) -> Self { Self(self.0.clone()) }
}

pub struct PuzzlePiece<T> {    
    graph: Arc<Mutex<PuzzleGraph>>,
    dependency: PuzzleDependency,
    answers: Answers<T>,
    pre_default: Arc<Mutex<Arc<dyn Fn() -> Option<T>>>>,
    post_default: Arc<Mutex<Arc<dyn Fn() -> Option<T>>>>,
    readies: Arc<Mutex<Vec<Box<dyn FnOnce(&mut PuzzlePiece<T>) + 'static>>>>,
    bid: u64
}

impl<T> Clone for PuzzlePiece<T> {
    fn clone(&self) -> Self {
        PuzzlePiece {
            graph: self.graph.clone(),
            dependency: self.dependency.clone(),
            answers: self.answers.clone(),
            pre_default: self.pre_default.clone(),
            post_default: self.post_default.clone(),
            readies: self.readies.clone(),
            bid: self.bid.clone()
        }
    }
}

impl<T: 'static> PuzzlePiece<T> {
    pub(super) fn new<F>(graph: &Arc<Mutex<PuzzleGraph>>, dependency: PuzzleDependency, default: F, bid: u64) -> PuzzlePiece<T> where F: Fn() -> Option<T> + 'static {
        PuzzlePiece {
            graph: graph.clone(),
            dependency,
            answers: Answers::new(bid),
            pre_default: Arc::new(Mutex::new(Arc::new(default))),
            post_default: Arc::new(Mutex::new(Arc::new(|| None))),
            readies: Arc::new(Mutex::new(vec![])),
            bid
        }
    }

    pub(super) fn erased(&self) -> Box<dyn ErasedPiece> { Box::new(self.clone()) }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize { self.answers.len() }

    pub fn set_answer(&self, solution: &mut PuzzleSolution, value: T) {
        let index = self.answers.set(value,solution.id());
        if !solution.set_answer_index(&self.dependency,&index) {
            /* double set: naughty user or default application */
            self.answers.finish(&index);
        } else {
            solution.just_answered().push(self.dependency.clone());
        }
    }

    pub fn add_solver<F>(&self, dependencies: &[PuzzleDependency], callback: F) where F: Fn(&mut PuzzleSolution) -> Option<T> + 'static {
        *lock!(self.post_default) = lock!(self.pre_default).clone();
        *lock!(self.pre_default) = Arc::new(|| None); // rely on solver
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

    #[cfg(debug_assertions)]
    pub fn set_name(&mut self, name: &str) { 
        self.dependency.set_name(name);
    }
}

impl<T: 'static> PuzzleValue<T> for PuzzlePiece<T> {
    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>> {
        let index = if let Some(x) = solution.get_answer_index(&self.dependency) { x } else { return None; };
        #[cfg(any(debug_assertions,test))]
        self.answers.check_for_aliens(solution.bid,&self.dependency.name());
        self.answers.get(&index)
    }

    fn dependency(&self) -> PuzzleDependency { self.dependency.clone() }

}

impl<T: 'static+ Clone> ClonablePuzzleValue<T> for PuzzlePiece<T> {}

impl<T: 'static> ErasedPiece for PuzzlePiece<T> {
    fn puzzle_ready(&mut self) {
        let readies = mem::replace(lock!(self.readies).as_mut(),vec![]);
        for ready in readies {
            ready(self);
        }
    }

    fn is_solved(&self, solution: &PuzzleSolution) -> bool {
        self.try_get(solution).is_some()
    }

    fn finish(&self, index: &AnswerIndex) {
        self.answers.finish(index);
    }

    fn apply_defaults(&mut self, solution: &mut PuzzleSolution, post: bool) {
        let ctor = if post { &self.post_default } else { &self.pre_default };
        if let Some(default) = (lock!(ctor))() {
            self.set_answer(solution,default);
        }
    }

    fn erased_dependency(&self) -> PuzzleDependency { self.dependency.clone() }
}

#[cfg(test)]
mod text {
    use crate::puzzle::puzzle::{Puzzle, PuzzleBuilder};

    use super::*;

    /* solver isbest tested at the puzzle level */
    
    #[test]
    fn piece_set() {
        let mut builder = PuzzleBuilder::new();
        let p1 = builder.new_piece();
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
        let p1 = builder.new_piece();
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
        let p1 : PuzzlePiece<()> = builder.new_piece();
        let flag2 = flag.clone();
        p1.add_ready(move |p| {
            *lock!(flag2) = true;
        });
        assert_eq!(false,*lock!(flag));
        let _puzzle = Puzzle::new(builder);
        assert_eq!(true,*lock!(flag));
    }
}
