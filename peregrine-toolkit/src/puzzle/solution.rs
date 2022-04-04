use std::sync::{Arc, Mutex};

use crate::{lock, log_extra, time::now };
use super::{answers::AnswerIndex, graph::PuzzleGraphReady, piece::ErasedPiece, Puzzle, solver::PuzzleSolver};
use lazy_static::lazy_static;
use identitynumber::{identitynumber, hashable};

identitynumber!(IDS);
hashable!(PuzzleSolution,id);

pub struct PuzzleSolution {
    pub bid: u64,
    id: u64,
    graph: PuzzleGraphReady,
    mapping: Vec<Option<AnswerIndex>>,
    pieces: Arc<Mutex<Vec<Box<dyn ErasedPiece>>>>,
    num_solved: usize,
    droppers: Vec<Box<dyn FnOnce() + 'static>>
}

impl PuzzleSolution {
    pub fn new(puzzle: &Puzzle) -> PuzzleSolution {
        let builder = puzzle.builder();
        PuzzleSolution {
            bid: builder.bid,
            id: IDS.next(),
            graph: PuzzleGraphReady::new(&builder,&*lock!(builder.graph)),
            mapping: vec![None;lock!(builder.pieces).len()],
            pieces: builder.pieces.clone(),
            num_solved: 0,
            droppers: vec![]
        }
    }

    pub fn id(&self) -> u64 { self.id }

    pub fn on_drop<F>(&mut self, dropper: F) where F: FnOnce() + 'static { 
        self.droppers.push(Box::new(dropper));
    }

    #[cfg(test)]
    pub(super) fn graph(&self) -> &PuzzleGraphReady { &self.graph }

    /* only pub(super) for testing */
    pub(super) fn all_solved(&self) -> bool { self.num_solved == self.mapping.len() }

    #[cfg(debug_assertions)]
    fn confess(&self) {
        use std::collections::HashSet;
        use crate::warn;

        let mut names = HashSet::new();
        for piece in lock!(self.pieces).iter() {
            if !piece.is_solved(self) {
                names.insert(piece.name().to_string());
            }
        }
        #[cfg(warn_missing_piece)]
        for name in &names {
            warn!("unsolved: {} ({})",name,self.id);
        }
    }

    #[allow(unused)]
    #[cfg(debug_assertions)]
    fn count(&self) {
        use std::collections::HashMap;

        use crate::warn;

        let mut counts = HashMap::new();
        for piece in lock!(self.pieces).iter() {
            *counts.entry(piece.name().to_string()).or_insert(0) += 1;
        }
        for (name,value) in &counts {
            warn!("count: {} {}",name,*value);
        }
    }

    fn solve_pre(&mut self) -> PuzzleSolver {
        let pieces = self.pieces.clone();
        for piece in lock!(pieces).iter_mut() {
            piece.apply_defaults(self,false);
        }
        PuzzleSolver::new(self,&self.graph)
    }

    fn solve_post(&mut self) -> bool {
        let pieces = self.pieces.clone();
        for piece in lock!(pieces).iter_mut() {
            piece.apply_defaults(self,true);
        }
        #[cfg(debug_assertions)]
        self.confess();
        #[cfg(debug_assertions)]
        self.count();
        log_extra!("{} pieces, {} solved id={}",self.mapping.len(),self.num_solved,self.id);
        self.all_solved()
    }

    pub fn solve(&mut self) -> bool {
        let from = now();
        let mut solver = self.solve_pre();
        while solver.run_one(self) {}        
        let out = self.solve_post();
        let took = now() - from;
        log_extra!("{} pieces, {} solved took {}ms id={}",self.mapping.len(),self.num_solved,took,self.id);
        out
    }

    pub(super) fn set_answer_index(&mut self, dependency: usize, index: &AnswerIndex) -> bool {
        if self.mapping[dependency].is_some() { return false; }
        self.num_solved += 1;
        self.mapping[dependency] = Some(index.clone());
        true
    }

    pub(super) fn get_answer_index(&self, dependency: usize) -> Option<AnswerIndex> {
        self.mapping[dependency].clone()
    }

    pub(super) fn is_solved(&self, dependency: &Option<usize>) -> bool { 
        let dependency_index = if let Some(index) = dependency { *index } else { return true; };
        self.mapping[dependency_index].is_some()
    }
    pub(super) fn num_pieces(&self) -> usize{ lock!(self.pieces).len() }
}

impl Drop for PuzzleSolution {
    fn drop(&mut self) {
        for dropper in self.droppers.drain(..) {
            dropper();
        }
        let pieces = lock!(self.pieces);
        for (dependency_index,answer_index) in self.mapping.iter().enumerate() {
            if let Some(answer_index) = answer_index {
                pieces[dependency_index].finish(answer_index);
            }
        }
    }
}

pub struct PuzzleMultiSolution {
    solutions: Vec<PuzzleSolution>
}

impl PuzzleMultiSolution {
    pub fn new(puzzle: &[Puzzle]) -> PuzzleMultiSolution {
        PuzzleMultiSolution {
            solutions: puzzle.iter().map(|p| PuzzleSolution::new(p)).collect()
        }
    }
    pub fn solve(&mut self) -> bool {
        let from = now();
        let mut solvers = self.solutions.iter_mut().map(|s| s.solve_pre()).collect::<Vec<_>>();
        loop {
            let mut any_changed = false;
            for (solution,solver) in self.solutions.iter_mut().zip(solvers.iter_mut()) {
                if solver.run_one(solution) {
                    any_changed = true;
                }
            }
            if !any_changed { break; }
        }
        let mut all_complete = true;
        for solution in &mut self.solutions {
            if !solution.solve_post() {
                all_complete = false;
            }
        }
        let took = now() - from;
        log_extra!("took {}ms",took);
        all_complete
    }
}
