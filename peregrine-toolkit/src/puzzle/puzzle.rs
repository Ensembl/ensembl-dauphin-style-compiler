use std::{sync::{Arc, Mutex}, borrow::Borrow};
use crate::lock;

use super::{piece::{PuzzlePiece}, graph::{PuzzleGraph, PuzzleSolver}, answers::{AnswerIndex}, piece::{ErasedPiece}};

#[cfg(test)]
use std::sync::MutexGuard;

#[cfg_attr(test,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub struct PuzzleDependency {
    index: Option<usize>
}

impl PuzzleDependency {
    fn new(index: usize) -> PuzzleDependency {
        PuzzleDependency { index: Some(index) }
    }

    pub(super) fn none() -> PuzzleDependency { PuzzleDependency { index: None }}

    pub(super) fn index(&self) -> Option<usize> { self.index }
}

#[derive(Clone)]
pub struct Puzzle {
    graph: Arc<Mutex<PuzzleGraph>>,
    pieces: Arc<Mutex<Vec<Box<dyn ErasedPiece>>>>
}

impl Puzzle {
    pub fn new() -> Puzzle {
        Puzzle {
            graph: Arc::new(Mutex::new(PuzzleGraph::new())),
            pieces: Arc::new(Mutex::new(vec![]))
        }
    }

    pub fn new_piece<T: 'static>(&self, default: Option<T>) -> PuzzlePiece<T> {
        let mut pieces = lock!(self.pieces);
        let id = pieces.len();
        let out = PuzzlePiece::new(&self.graph,PuzzleDependency::new(id),default);
        pieces.push(out.erased());
        out
    }
}

pub struct PuzzleSolution {
    graph: Arc<Mutex<PuzzleGraph>>,
    mapping: Vec<Option<AnswerIndex>>,
    pieces: Arc<Mutex<Vec<Box<dyn ErasedPiece>>>>,
    just_answered: Vec<PuzzleDependency>,
    num_solved: usize
}

impl PuzzleSolution {
    pub fn new(puzzle: &Puzzle) -> PuzzleSolution {
        PuzzleSolution {
            graph: puzzle.graph.clone(),
            mapping: vec![None;lock!(puzzle.pieces).len()],
            pieces: puzzle.pieces.clone(),
            just_answered: vec![],
            num_solved: 0
        }
    }

    #[cfg(test)]
    pub(super) fn graph(&self) -> MutexGuard<PuzzleGraph> { lock!(self.graph) }

    /* only pub(super) for testing */
    pub(super) fn all_solved(&self) -> bool { self.num_solved == self.mapping.len() }

    pub fn solve(&mut self) -> bool {
        let pieces = self.pieces.clone();
        for piece in lock!(pieces).iter() {
            piece.apply_defaults(self);
        }
        let mut solver = PuzzleSolver::new(self,lock!(self.graph).borrow());
        solver.run(self);
        self.all_solved()
    }

    pub(super) fn set_answer_index(&mut self, dependency: &PuzzleDependency, index: &AnswerIndex) -> bool {
        let dependency_index = if let Some(index) = dependency.index { index } else { return false; };
        if self.mapping[dependency_index].is_some() { return false; }
        self.num_solved += 1;
        self.mapping[dependency_index] = Some(index.clone());
        true
    }

    pub(super) fn get_answer_index(&self, dependency: &PuzzleDependency) -> Option<AnswerIndex> {
        let dependency_index = if let Some(index) = dependency.index { index } else { return None; };
        self.mapping[dependency_index].clone()
    }

    pub(super) fn is_solved(&self, dependency: &PuzzleDependency) -> bool { 
        let dependency_index = if let Some(index) = dependency.index { index } else { return true; };
        self.mapping[dependency_index].is_some()
    }
    pub(super) fn num_pieces(&self) -> usize{ lock!(self.pieces).len() }
    pub(super) fn just_answered(&mut self) -> &mut Vec<PuzzleDependency> { &mut self.just_answered }
}

impl Drop for PuzzleSolution {
    fn drop(&mut self) {
        let pieces = lock!(self.pieces);
        for (dependency_index,answer_index) in self.mapping.iter().enumerate() {
            if let Some(answer_index) = answer_index {
                pieces[dependency_index].finish(answer_index);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::puzzle::piece::{PuzzleValue, ClonablePuzzleValue};

    use super::*;

    // XXX fixed answers
    #[test]
    fn puzzle_smoke() {
        let mut puzzle = Puzzle::new();
        for order in 0..6 {
            let q1 = puzzle.new_piece(None);
            let q2 = puzzle.new_piece(None);
            let q3 = puzzle.new_piece(None);
            let (p1,p2,p3) = match order {
                0 => (q1,q2,q3),
                1 => (q1,q3,q2),
                2 => (q2,q1,q3),
                3 => (q2,q3,q1),
                4 => (q3,q1,q2),
                _ => (q3,q2,q1),
            };
            let p1b = p1.clone();
            p2.add_solver(&[p1.dependency()], move |solution| {
                Some(p1b.get_clone(solution) + 2)
            });
            let p1b = p1.clone();
            let p2b = p2.clone();
            p3.add_solver(&[p1.dependency(),p2.dependency()], move |solution| {
                Some(p1b.get_clone(solution) + p2b.get_clone(solution))
            });
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
    }

    #[test]
    fn puzzle_drop() {
        let mut puzzle = Puzzle::new();
        let p1 = puzzle.new_piece(None);
        let p2 = puzzle.new_piece(None);
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
        let mut puzzle = Puzzle::new();
        let p1 = puzzle.new_piece(Some(7));
        let p2 = puzzle.new_piece(None);
        let p3 = puzzle.new_piece(None);
        let p1b = p1.clone();
        p2.add_solver(&[p1.dependency()], move |solution| {
            Some(p1b.get_clone(solution) + 2)
        });
        let p1b = p1.clone();
        let p2b = p2.clone();
        p3.add_solver(&[p1.dependency(),p2.dependency()], move |solution| {
            Some(p1b.get_clone(solution) + p2b.get_clone(solution))
        });
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
