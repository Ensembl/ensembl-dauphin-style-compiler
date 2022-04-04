use std::{sync::{Arc, Mutex}, mem};

use crate::{ lock, error };

use super::{puzzle::{PuzzleDependency}, graph::PuzzleGraph, answers::{Answers, AnswerIndex}, PuzzleBuilder, PuzzleSolution,};

pub(super) trait ErasedPiece {
    fn puzzle_ready(&mut self, builder: &PuzzleBuilder);
    fn finish(&self, index: &AnswerIndex);
    fn apply_defaults(&mut self, solution: &mut PuzzleSolution, post: bool);
    fn is_solved(&self, solution: &PuzzleSolution) -> bool;

    #[cfg(debug_assertions)]
    fn name(&self) -> String;
}

pub trait PuzzleValue<T: 'static> {
    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>>;
    fn get(&self, solution: &PuzzleSolution) -> Arc<T> { self.try_get(solution).unwrap() }
    fn known_constant_value(&self) -> Option<Arc<T>>;
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
    fn known_constant_value(&self) -> Option<Arc<T>> { self.0.known_constant_value() }
}

impl<T: 'static+ Clone> ClonablePuzzleValue<T> for PuzzleValueHolder<T> {}

impl<T: 'static> Clone for PuzzleValueHolder<T> {
    fn clone(&self) -> Self { Self(self.0.clone()) }
}

/* Convenience class for PuzzlePiece and PuzzleCombination commonalities */
struct PuzzleVariable<T> {
    graph: Arc<Mutex<PuzzleGraph>>,
    dependency: usize,
    answers: Answers<T>,
    bid: u64,

    #[cfg(debug_assertions)]
    name: Arc<Mutex<String>>
}

impl<T> PuzzleVariable<T> {
    pub(super) fn new(graph: &Arc<Mutex<PuzzleGraph>>, dependency: usize, bid: u64) -> PuzzleVariable<T> {
        PuzzleVariable {
            graph: graph.clone(),
            dependency,
            answers: Answers::new(bid),
            bid,

            #[cfg(debug_assertions)]
            name: Arc::new(Mutex::new("".to_string()))
        }
    }

    pub fn set_answer(&self, solution: &mut PuzzleSolution, value: T) {
        let index = self.answers.set(value);
        if !solution.set_answer_index(self.dependency,&index) {
            /* double set: naughty user or default application */
            self.answers.finish(&index);
        }
    }
}

impl<T> Clone for PuzzleVariable<T> {
    fn clone(&self) -> Self {
        PuzzleVariable {
            graph: self.graph.clone(),
            dependency: self.dependency.clone(),
            answers: self.answers.clone(),
            bid: self.bid.clone(),

            #[cfg(debug_assertions)]
            name: self.name.clone()
        }
    }
}

impl<T: 'static> PuzzleValue<T> for PuzzleVariable<T> {
    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>> {
        let index = solution.get_answer_index(self.dependency);
        #[cfg(any(debug_assertions,test))]
        self.answers.check_for_aliens(solution.bid,&lock!(self.name));
        index.and_then(|index| self.answers.get(&index))
    }

    #[cfg(debug_assertions)]
    fn get(&self, solution: &PuzzleSolution) -> Arc<T> {
        let out = self.try_get(solution);
        if out.is_none() {
            error!("get on undefined: {}",lock!(self.name));
        }
        out.unwrap()
    }

    fn known_constant_value(&self) -> Option<Arc<T>> { None }
    fn dependency(&self) -> PuzzleDependency { PuzzleDependency::variable(self.dependency) }
}

pub struct PuzzlePiece<T> { 
    variable: PuzzleVariable<T>,
    readies: Arc<Mutex<Vec<Box<dyn FnOnce(&mut PuzzlePiece<T>,&PuzzleBuilder) + 'static>>>>,
    pre_default: Arc<Mutex<Arc<dyn Fn() -> Option<T>>>>,
}

impl<T> Clone for PuzzlePiece<T> {
    fn clone(&self) -> Self {
        PuzzlePiece {
            variable: self.variable.clone(),
            readies: self.readies.clone(),
            pre_default: self.pre_default.clone(),
        }
    }
}

impl<T: 'static> PuzzlePiece<T> {
    pub(super) fn new<F>(graph: &Arc<Mutex<PuzzleGraph>>, dependency: usize, default: F, bid: u64) -> PuzzlePiece<T> where F: Fn() -> Option<T> + 'static {
        let out = PuzzlePiece {
            variable: PuzzleVariable::new(graph,dependency,bid),
            readies: Arc::new(Mutex::new(vec![])),
            pre_default: Arc::new(Mutex::new(Arc::new(default))),
        };
        lock!(out.variable.graph).add_known(&PuzzleDependency::variable(out.variable.dependency));
        out
    }

    #[cfg(debug_assertions)]
    fn get(&self, solution: &PuzzleSolution) -> Arc<T> { self.variable.get(solution) }

    pub(super) fn erased(&self) -> Box<dyn ErasedPiece> { Box::new(self.clone()) }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize { self.variable.answers.len() }

    pub fn set_answer(&self, solution: &mut PuzzleSolution, value: T) {
        self.variable.set_answer(solution,value);
    }

    pub fn add_ready<F>(&self, cb: F) where F: FnOnce(&mut PuzzlePiece<T>,&PuzzleBuilder) + 'static {
        lock!(self.readies).push(Box::new(cb))
    }

    #[cfg(debug_assertions)]
    pub fn set_name(&mut self, name: &str) { 
        *lock!(self.variable.name) = name.to_string();
    }
}

impl<T: 'static> PuzzleValue<T> for PuzzlePiece<T> {
    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>> { self.variable.try_get(solution) }

    #[cfg(debug_assertions)]
    fn get(&self, solution: &PuzzleSolution) -> Arc<T> { self.variable.get(solution) }
    fn known_constant_value(&self) -> Option<Arc<T>> { self.variable.known_constant_value() }
    fn dependency(&self) -> PuzzleDependency { self.variable.dependency() }
}

impl<T: 'static+ Clone> ClonablePuzzleValue<T> for PuzzlePiece<T> {}

impl<T: 'static> ErasedPiece for PuzzlePiece<T> {
    fn puzzle_ready(&mut self, builder: &PuzzleBuilder) {
        let readies = mem::replace(lock!(self.readies).as_mut(),vec![]);
        for ready in readies {
            ready(self,builder);
        }
    }

    fn is_solved(&self, solution: &PuzzleSolution) -> bool {
        self.try_get(solution).is_some()
    }

    fn finish(&self, index: &AnswerIndex) { self.variable.answers.finish(index); }

    fn apply_defaults(&mut self, solution: &mut PuzzleSolution, post: bool) {
        if !post {
            if let Some(default) = (lock!(&self.pre_default))() {
                self.set_answer(solution,default);
            }
        }
    }

    #[cfg(debug_assertions)]
    fn name(&self) -> String { lock!(self.variable.name).to_string() }
}

pub struct PuzzleCombination<T> { 
    variable: PuzzleVariable<T>, 
    post_default: Arc<Mutex<Arc<dyn Fn() -> Option<T>>>>,
    readies: Arc<Mutex<Vec<Box<dyn FnOnce(&mut PuzzleCombination<T>,&PuzzleBuilder) + 'static>>>>,
}

impl<T> Clone for PuzzleCombination<T> {
    fn clone(&self) -> Self {
        PuzzleCombination {
            variable: self.variable.clone(),
            post_default: self.post_default.clone(),
            readies: self.readies.clone(),
        }
    }
}

impl<T: 'static> PuzzleCombination<T> {
    pub(super) fn new<F,G>(graph: &Arc<Mutex<PuzzleGraph>>, dependency: usize, default: F, bid: u64, dependencies: &[PuzzleDependency], callback: G) -> PuzzleCombination<T> where F: Fn() -> Option<T> + 'static, G: Fn(&mut PuzzleSolution) -> T + 'static {
        let out = PuzzleCombination {
            variable: PuzzleVariable::new(graph,dependency,bid),
            post_default: Arc::new(Mutex::new(Arc::new(default))),
            readies: Arc::new(Mutex::new(vec![])),
        };
        out.add_solver(dependencies,callback);
        out
    }

    #[cfg(debug_assertions)]
    fn get(&self, solution: &PuzzleSolution) -> Arc<T> { self.variable.get(solution) }

    pub(super) fn erased(&self) -> Box<dyn ErasedPiece> { Box::new(self.clone()) }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize { self.variable.answers.len() }

    pub fn set_answer(&self, solution: &mut PuzzleSolution, value: T) {
        self.variable.set_answer(solution,value);
    }

    fn add_solver<F>(&self, dependencies: &[PuzzleDependency], callback: F) where F: Fn(&mut PuzzleSolution) -> T + 'static {
        let self2 = self.clone();
        /* Do all this in nesting callback to avoid polymorphism infecting solver */
        let solver = move |solution: &mut PuzzleSolution| {
            let answer = callback(solution);
            self2.set_answer(solution,answer);
        };
        lock!(self.variable.graph).add_solver(&PuzzleDependency::variable(self.variable.dependency),dependencies,Arc::new(solver));
    }

    pub fn add_ready<F>(&self, cb: F) where F: FnOnce(&mut PuzzleCombination<T>,&PuzzleBuilder) + 'static {
        lock!(self.readies).push(Box::new(cb))
    }

    #[cfg(debug_assertions)]
    pub fn set_name(&mut self, name: &str) {  *lock!(self.variable.name) = name.to_string(); }
}

impl<T: 'static> PuzzleValue<T> for PuzzleCombination<T> {
    fn try_get(&self, solution: &PuzzleSolution) -> Option<Arc<T>> { self.variable.try_get(solution) }

    #[cfg(debug_assertions)]
    fn get(&self, solution: &PuzzleSolution) -> Arc<T> { self.variable.get(solution) }
    fn known_constant_value(&self) -> Option<Arc<T>> { self.variable.known_constant_value() }
    fn dependency(&self) -> PuzzleDependency { self.variable.dependency() }
}

impl<T: 'static+ Clone> ClonablePuzzleValue<T> for PuzzleCombination<T> {}

impl<T: 'static> ErasedPiece for PuzzleCombination<T> {
    fn puzzle_ready(&mut self, builder: &PuzzleBuilder) {
        let readies = mem::replace(lock!(self.readies).as_mut(),vec![]);
        for ready in readies {
            ready(self,builder);
        }
    }

    fn is_solved(&self, solution: &PuzzleSolution) -> bool {
        self.try_get(solution).is_some()
    }

    fn finish(&self, index: &AnswerIndex) {
        self.variable.answers.finish(index);
    }

    fn apply_defaults(&mut self, solution: &mut PuzzleSolution, post: bool) {
        if post {
            if let Some(default) = (lock!(self.post_default))() {
                self.set_answer(solution,default);
            }
        }
    }

    #[cfg(debug_assertions)]
    fn name(&self) -> String { lock!(self.variable.name).to_string() }
}


#[cfg(test)]
mod text {
    use crate::puzzle::puzzle::{Puzzle, PuzzleBuilder};

    use super::*;

    /* solver isbest tested at the puzzle level */
    
    #[test]
    fn piece_set() {
        let builder = PuzzleBuilder::new();
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
        let builder = PuzzleBuilder::new();
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
        let builder = PuzzleBuilder::new();
        let flag = Arc::new(Mutex::new(false));
        let p1 : PuzzlePiece<()> = builder.new_piece();
        let flag2 = flag.clone();
        p1.add_ready(move |_,_| {
            *lock!(flag2) = true;
        });
        assert_eq!(false,*lock!(flag));
        let _puzzle = Puzzle::new(builder);
        assert_eq!(true,*lock!(flag));
    }
}
