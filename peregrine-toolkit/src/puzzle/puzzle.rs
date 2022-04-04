use std::{sync::{Arc, Mutex}, collections::{HashSet,HashMap}, mem };
use crate::{lock, log_extra, time::now};

#[cfg(debug_assertions)]
#[allow(unused)]
use crate::warn;

use super::{piece::{PuzzlePiece, PuzzleCombination}, graph::{PuzzleGraph, PuzzleGraphReady}, answers::{AnswerIndex}, piece::{ErasedPiece}, PuzzleSolution};

use lazy_static::lazy_static;
use identitynumber::{identitynumber, hashable};

#[cfg_attr(any(test,debug_assertions),derive(Debug,PartialEq,Eq))]
#[derive(Clone)]
enum PuzzleDependencyValue {
    Constant,
    Variable(usize),
    Delayed(DelaySlot)
}

#[cfg_attr(any(test,debug_assertions),derive(Debug))]
#[derive(Clone)]
pub struct PuzzleDependency {
    index: PuzzleDependencyValue,
    #[cfg(debug_assertions)]
    name: Arc<Mutex<String>>
}

#[cfg(test)]
impl PartialEq for PuzzleDependency {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl PuzzleDependency {
    pub(super) fn variable(index: usize) -> PuzzleDependency {
        PuzzleDependency {
            index: PuzzleDependencyValue::Variable(index),
            #[cfg(debug_assertions)]
            name: Arc::new(Mutex::new("".to_string()))
         }
    }

    #[cfg(test)]
    pub(super) fn partial_resolve(&self) -> Option<usize> {
        match &self.index {
            PuzzleDependencyValue::Constant => None,
            PuzzleDependencyValue::Variable(x) => Some(*x),
            PuzzleDependencyValue::Delayed(_) => None
        }
    }

    #[cfg(debug_assertions)]
    pub fn name(&self) -> String { lock!(self.name).clone() }

    #[cfg(debug_assertions)]
    pub fn set_name(&mut self, name: &str) { *lock!(self.name) = name.to_string(); }

    pub fn constant() -> PuzzleDependency {
        PuzzleDependency {
             index: PuzzleDependencyValue::Constant,
             #[cfg(debug_assertions)]
             name: Arc::new(Mutex::new("".to_string()))
        }
    }

    pub(super) fn delayed(slot: &DelaySlot) -> PuzzleDependency {
        PuzzleDependency {
             index: PuzzleDependencyValue::Delayed(slot.clone()),
             #[cfg(debug_assertions)]
             name: Arc::new(Mutex::new("".to_string()))
        }
    }

    pub(super) fn resolve(&self, builder: &PuzzleBuilder) -> Option<usize> {
        match &self.index {
            PuzzleDependencyValue::Constant => None,
            PuzzleDependencyValue::Variable(x) => Some(*x),
            PuzzleDependencyValue::Delayed(slot) => builder.get_delayed(&slot).resolve(builder)
        }
    }
}

#[cfg_attr(any(test,debug_assertions),derive(Debug,PartialEq,Eq))]
#[derive(Clone)]
pub(super) struct DelaySlot(usize);

identitynumber!(BIDS);

#[derive(Clone)]
pub struct PuzzleBuilder {
    pub bid: u64,
    readies: Arc<Mutex<Vec<Box<dyn FnOnce(&mut PuzzleBuilder) + 'static>>>>,
    pub(super) graph: Arc<Mutex<PuzzleGraph>>,
    pub(super) pieces: Arc<Mutex<Vec<Box<dyn ErasedPiece>>>>,
    delayed: Arc<Mutex<Vec<Option<PuzzleDependency>>>>
}

impl PuzzleBuilder {
    pub fn new() -> PuzzleBuilder {
        PuzzleBuilder {
            bid: BIDS.next(),
            graph: Arc::new(Mutex::new(PuzzleGraph::new())),
            pieces: Arc::new(Mutex::new(vec![])),
            readies: Arc::new(Mutex::new(vec![])),
            delayed: Arc::new(Mutex::new(vec![])),
        }
    }

    // XXX should be mut
    pub fn new_piece<T: 'static>(&self) -> PuzzlePiece<T> {
        let mut pieces = lock!(self.pieces);
        let id = pieces.len();
        let out = PuzzlePiece::new(&self.graph,id,|| None,self.bid);
        pieces.push(out.erased());
        out
    }

    // XXX should be mut
    pub fn new_piece_default<T: Clone+'static>(&self, default: T) -> PuzzlePiece<T> {
        let mut pieces = lock!(self.pieces);
        let id = pieces.len();
        let out = PuzzlePiece::new(&self.graph,id,move || Some(default.clone()),self.bid);
        pieces.push(out.erased());
        out
    }

    pub fn new_combination<F,T: 'static>(&self,dependencies: &[PuzzleDependency], callback: F) -> PuzzleCombination<T> where F: Fn(&mut PuzzleSolution) -> T + 'static {
        let mut pieces = lock!(self.pieces);
        let id = pieces.len();
        let out = PuzzleCombination::new(&self.graph,id,|| None,self.bid,dependencies,callback);
        pieces.push(out.erased());
        out
    }

    // XXX should be mut
    pub fn new_combination_default<F,T: Clone+'static>(&self, default: T,dependencies: &[PuzzleDependency], callback: F) -> PuzzleCombination<T> where F: Fn(&mut PuzzleSolution) -> T + 'static {
        let mut pieces = lock!(self.pieces);
        let id = pieces.len();
        let out = PuzzleCombination::new(&self.graph,id,move || Some(default.clone()),self.bid,dependencies,callback);
        pieces.push(out.erased());
        out
    }

    pub fn add_ready<F>(&self, cb: F) where F: FnOnce(&mut PuzzleBuilder) + 'static {
        lock!(self.readies).push(Box::new(cb));
    }

    fn run_readies(&mut self) {
        let readies = mem::replace(&mut *lock!(self.readies),vec![]);
        for ready in readies {
            ready(self);
        }
        for piece in lock!(self.pieces).iter_mut() {
            piece.puzzle_ready(self);
        }
    }

    pub(super) fn allocate_delayed(&self) -> DelaySlot {
        let mut delayed = lock!(self.delayed);
        let index = delayed.len();
        delayed.push(None);
        DelaySlot(index)
    }

    pub(super) fn set_delayed(&self, slot: &DelaySlot, value: PuzzleDependency) {
        let mut delayed = lock!(self.delayed);
        delayed[slot.0] = Some(value);
    }

    pub(super) fn get_delayed(&self, slot: &DelaySlot) -> PuzzleDependency {
        let delayed = lock!(self.delayed);
        if delayed[slot.0].is_none() {
            panic!("delayed slot not populated");
        }
        delayed[slot.0].as_ref().unwrap().clone()
    }
}

#[derive(Clone)]
pub struct Puzzle(Arc<PuzzleBuilder>);

impl Puzzle {
    pub fn new(mut builder: PuzzleBuilder) -> Puzzle {
        builder.run_readies();
        let out = Puzzle(Arc::new(builder));
        out
    }

    pub(super) fn builder(&self) -> &PuzzleBuilder { &self.0 }
    pub fn bid(&self) -> u64 { self.0.bid }
}

#[cfg(test)]
mod test {
    use crate::puzzle::{piece::{PuzzleValue, ClonablePuzzleValue}, PuzzleSolution};

    use super::*;

    // XXX fixed answers
    #[test]
    fn puzzle_smoke() {
        let builder = PuzzleBuilder::new();
        let p1 = builder.new_piece();
        let p1b = p1.clone();
        let p2 = builder.new_combination(&[p1.dependency()], move |solution| {
            p1b.get_clone(solution) + 2
        });
        let p1b = p1.clone();
        let p2b = p2.clone();
        let p3 = builder.new_combination(&[p1.dependency(),p2.dependency()], move |solution| {
            p1b.get_clone(solution) + p2b.get_clone(solution)
        });
        let puzzle = Puzzle::new(builder);
        let mut s1 = PuzzleSolution::new(&puzzle);
        let mut s2 = PuzzleSolution::new(&puzzle);
        p1.set_answer(&mut s1,2);
        p1.set_answer(&mut s2,3);
        s1.solve();
        s2.solve();
        assert_eq!(Some(2),p1.try_get_clone(&s1));
        assert_eq!(Some(4),p2.try_get_clone(&s1));
        assert_eq!(Some(6),p3.try_get_clone(&s1));
        assert_eq!(Some(3),p1.try_get_clone(&s2));
        assert_eq!(Some(5),p2.try_get_clone(&s2));
        assert_eq!(Some(8),p3.try_get_clone(&s2));
    }

    #[test]
    fn puzzle_drop() {
        let builder = PuzzleBuilder::new();
        let p1 = builder.new_piece();
        let p2 = builder.new_piece();
        let puzzle = Puzzle::new(builder);
        let mut s1 = PuzzleSolution::new(&puzzle);      
        let mut s2 = PuzzleSolution::new(&puzzle);      
        let mut s3 = PuzzleSolution::new(&puzzle);
        p1.set_answer(&mut s3, 103);
        p1.set_answer(&mut s1, 101);
        p2.set_answer(&mut s2, 202);
        p1.set_answer(&mut s2, 102);
        p2.set_answer(&mut s1, 201);
        p2.set_answer(&mut s3, 203);
        assert_eq!(Some(101),p1.try_get_clone(&s1));
        assert_eq!(Some(102),p1.try_get_clone(&s2));
        assert_eq!(Some(103),p1.try_get_clone(&s3));
        assert_eq!(Some(201),p2.try_get_clone(&s1));
        assert_eq!(Some(202),p2.try_get_clone(&s2));
        assert_eq!(Some(203),p2.try_get_clone(&s3));
        drop(s2);
        assert_eq!(2,p1.len());
        assert_eq!(3,p2.len());
        drop(s1);
        assert_eq!(1,p1.len());
        assert_eq!(3,p2.len());
        drop(s3);
        assert_eq!(0,p1.len());
        assert_eq!(0,p2.len());
    }

    #[test]
    fn puzzle_default() {
        let builder = PuzzleBuilder::new();
        let p1 = builder.new_piece_default(7);
        let p1b = p1.clone();
        let p2 = builder.new_combination(&[p1.dependency()], move |solution| {
            p1b.get_clone(solution) + 2
        });
        let p1b = p1.clone();
        let p2b = p2.clone();
        let p3 = builder.new_combination(&[p1.dependency(),p2.dependency()], move |solution| {
            p1b.get_clone(solution) + p2b.get_clone(solution)
        });
        let puzzle = Puzzle::new(builder);
        let mut s1 = PuzzleSolution::new(&puzzle);
        let mut s2 = PuzzleSolution::new(&puzzle);
        p1.set_answer(&mut s2,3);
        s1.solve();
        s2.solve();
        assert_eq!(Some(7),p1.try_get_clone(&s1));
        assert_eq!(Some(9),p2.try_get_clone(&s1));
        assert_eq!(Some(16),p3.try_get_clone(&s1));
        assert_eq!(Some(3),p1.try_get_clone(&s2));
        assert_eq!(Some(5),p2.try_get_clone(&s2));
        assert_eq!(Some(8),p3.try_get_clone(&s2));
    }
}
