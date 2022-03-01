use std::{sync::Arc, mem};

#[cfg(test)]
use std::fmt;

use super::puzzle::{PuzzleSolution, PuzzleDependency};

#[derive(Clone)]
struct PuzzleGraphNode {
    target: PuzzleDependency,
    sources: Vec<PuzzleDependency>,
    callback: Arc<dyn Fn(&mut PuzzleSolution)>
}

#[cfg(test)]
impl fmt::Debug for PuzzleGraphNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PuzzleGraphNode").field("target", &self.target).field("sources", &self.sources).finish()
    }
}

pub(super) struct PuzzleGraph {
    nodes: Vec<PuzzleGraphNode>
}

impl PuzzleGraph {
    pub(super) fn new() -> PuzzleGraph {
        PuzzleGraph {
            nodes: vec![]
        }
    }

    pub(super) fn add_solver(&mut self, target: &PuzzleDependency, sources: &[PuzzleDependency], callback: Arc<dyn Fn(&mut PuzzleSolution)>) {
        self.nodes.push(PuzzleGraphNode {
            target: target.clone(),
            sources: sources.to_vec(),
            callback
        })
    }
}

pub(super) struct PuzzleSolver {
    ready: Vec<PuzzleGraphNode>,
    pending: Vec<Vec<PuzzleGraphNode>>
}

impl PuzzleSolver {
    fn add_dependency(&mut self, solution: &PuzzleSolution, mut node: PuzzleGraphNode) {
        while let Some(dependency) = node.sources.pop() {
            if !solution.is_solved(&dependency) {
                if let Some(dependency_index) = dependency.index() {
                    self.pending[dependency_index].push(node.clone());
                }
                return;
            }
        }
        self.ready.push(node.clone());
    }

    pub(super) fn new(solution: &PuzzleSolution, graph: &PuzzleGraph) -> PuzzleSolver {
        let num_pieces = solution.num_pieces();
        let mut out = PuzzleSolver {
            ready: vec![],
            pending: vec![vec![];num_pieces]
        };
        for node in &graph.nodes {
            out.add_dependency(solution,node.clone());
        }
        out
    }

    fn remove(&mut self, solution: &PuzzleSolution, target: &PuzzleDependency) {
        if let Some(target_index) = target.index() {
            let freed = mem::replace(&mut self.pending[target_index],vec![]);
            for node in freed {
                self.add_dependency(solution,node);
            }
        }
    }

    fn run_one(&mut self, solution: &mut PuzzleSolution) -> bool {
        let answered = mem::replace(solution.just_answered(),vec![]);
        for answered in answered {
            self.remove(solution,&answered);
        }
        let node = if let Some(n) = self.ready.pop() { n } else { return false };
        if !solution.is_solved(&node.target) {
            (node.callback)(solution);
        }
        true
    }

    pub(super) fn run(&mut self, solution: &mut PuzzleSolution) {
        while self.run_one(solution) {}
    }
}

#[cfg(test)]
mod test {
    use crate::puzzle::{puzzle::Puzzle, piece::{PuzzlePiece, PuzzleValue, ClonablePuzzleValue}, PuzzleBuilder};

    use super::*;

    struct Setup {
        s1: PuzzleSolution,
        p1: PuzzlePiece<i32>,
        p2: PuzzlePiece<i32>,
        p3: PuzzlePiece<i32>,
    }

    fn setup(p2_solver: bool) -> Setup {
        let mut builder = PuzzleBuilder::new();
        let p3 = builder.new_piece(None);
        let p1 : PuzzlePiece<i32> = builder.new_piece(None);
        let p2 = builder.new_piece(None);
        if p2_solver {
            let p1b = p1.clone();
            p2.add_solver(&[p1.dependency()], move |solution| {
                Some(p1b.get_clone(solution) + 2)
            });
        }
        let p1b = p1.clone();
        let p2b = p2.clone();
        p3.add_solver(&[p1.dependency(),p2.dependency()], move |solution| {
            Some(p1b.get_clone(solution) + p2b.get_clone(solution))
        });
        let puzzle = Puzzle::new(builder);
        let s1 = PuzzleSolution::new(&puzzle);
        Setup { s1, p1, p2, p3 }
    }

    #[test]
    fn graph_smoke() {
        let setup = setup(true);
        let graph = setup.s1.graph();
        assert_eq!(2,graph.nodes.len());
        assert_eq!(setup.p2.dependency(),graph.nodes[0].target);
        assert_eq!(vec![setup.p1.dependency()],graph.nodes[0].sources);
        assert_eq!(setup.p3.dependency(),graph.nodes[1].target);
        assert_eq!(vec![setup.p1.dependency(),setup.p2.dependency()],graph.nodes[1].sources);
    }

    #[test]
    fn solver_smoke() {
        let mut setup = setup(true);
        let graph = setup.s1.graph();
        let mut solver = PuzzleSolver::new(&setup.s1,&graph);
        drop(graph);
        assert_eq!(0,solver.ready.len());
        setup.p1.set_answer(&mut setup.s1,1);
        assert!(solver.run_one(&mut setup.s1));
        assert_eq!(1,setup.s1.just_answered().len());
        assert_eq!(setup.p2.dependency(),setup.s1.just_answered()[0]);
        assert!(solver.run_one(&mut setup.s1));
        assert_eq!(1,setup.s1.just_answered().len());
        assert_eq!(setup.p3.dependency(),setup.s1.just_answered()[0]);
        assert!(!solver.run_one(&mut setup.s1));
        assert!(setup.s1.all_solved());
    }

    #[test]
    fn solver_steps_switch() {
        let mut setup = setup(false);
        let graph = setup.s1.graph();
        let mut solver = PuzzleSolver::new(&setup.s1,&graph);
        drop(graph);
        assert_eq!(0,solver.ready.len());
        assert_eq!(0,solver.pending[1].len());
        assert_eq!(1,solver.pending[2].len());
        setup.p2.set_answer(&mut setup.s1,2);
        assert!(!solver.run_one(&mut setup.s1));
        println!("{:?}",solver.pending);
        assert_eq!(1,solver.pending[1].len());
        assert_eq!(0,solver.pending[2].len());
        setup.p1.set_answer(&mut setup.s1,1);
        assert!(!setup.s1.all_solved());
        assert!(solver.run_one(&mut setup.s1));
        assert_eq!(0,solver.pending[1].len());
        assert_eq!(0,solver.pending[2].len());
        assert!(setup.s1.all_solved());
        assert!(!solver.run_one(&mut setup.s1));
    }

    #[test]
    fn solver_steps_no_switch() {
        let mut setup = setup(false);
        let graph = setup.s1.graph();
        let mut solver = PuzzleSolver::new(&setup.s1,&graph);
        drop(graph);
        assert_eq!(0,solver.ready.len());
        assert_eq!(0,solver.pending[1].len());
        assert_eq!(1,solver.pending[2].len());
        setup.p1.set_answer(&mut setup.s1,1);
        assert!(!solver.run_one(&mut setup.s1));
        println!("{:?}",solver.pending);
        assert_eq!(0,solver.pending[1].len());
        assert_eq!(1,solver.pending[2].len());
        setup.p2.set_answer(&mut setup.s1,2);
        assert!(!setup.s1.all_solved());
        assert!(solver.run_one(&mut setup.s1));
        assert_eq!(0,solver.pending[1].len());
        assert_eq!(0,solver.pending[2].len());
        assert!(setup.s1.all_solved());
        assert!(!solver.run_one(&mut setup.s1));
    }

    #[test]
    fn solver_unsolvable() {
        let mut builder = PuzzleBuilder::new();
        let p3 = builder.new_piece(None);
        let p1 : PuzzlePiece<i32> = builder.new_piece(None);
        let p2 = builder.new_piece(None);
        p2.add_solver(&[p1.dependency(),p3.dependency()], move |_| Some(0));
        p3.add_solver(&[p1.dependency(),p2.dependency()], move |_| Some(0));
        let puzzle = Puzzle::new(builder);
        let mut s1 = PuzzleSolution::new(&puzzle);
        p1.set_answer(&mut s1,1);
        assert!(!s1.solve());
    }

    #[test]
    fn solver_solvable() {
        let mut builder = PuzzleBuilder::new();
        let p3 = builder.new_piece(None);
        let p1 : PuzzlePiece<i32> = builder.new_piece(None);
        let p2 = builder.new_piece(None);
        p2.add_solver(&[p1.dependency(),p3.dependency()], move |_| Some(0));
        p3.add_solver(&[p1.dependency()], move |_| Some(0));
        let puzzle = Puzzle::new(builder);
        let mut s1 = PuzzleSolution::new(&puzzle);
        p1.set_answer(&mut s1,1);
        assert!(s1.solve());
    }
}
