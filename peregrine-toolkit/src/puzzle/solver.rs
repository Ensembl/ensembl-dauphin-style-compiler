use std::{mem, sync::Arc};

#[cfg(any(test,debug_assertions))]
use std::fmt;

use crate::log;

use super::{PuzzleSolution, graph::{PuzzleGraphReady}, toposort::TopoSort};

#[derive(Clone)]
pub(super) struct PuzzleSolverNode {
    target: Option<usize>,
    sources: Vec<Option<usize>>,
    callback: Arc<dyn Fn(&mut PuzzleSolution)>
}

#[cfg(any(test,debug_assertions))]
impl fmt::Debug for PuzzleSolverNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PuzzleSolverNode").field("target", &self.target).field("sources", &self.sources).finish()
    }
}

impl PuzzleSolverNode {
    pub(super) fn new(target: Option<usize>, sources: Vec<Option<usize>>, callback: Arc<dyn Fn(&mut PuzzleSolution)>) -> PuzzleSolverNode {
        PuzzleSolverNode { target, sources, callback }
    }
}

pub(super) struct PuzzleSolver {
    nodes: Vec<PuzzleSolverNode>,
    order: Vec<usize>,
    index: usize
}

impl PuzzleSolver {
    pub(super) fn new(solution: &PuzzleSolution, graph: &PuzzleGraphReady) -> PuzzleSolver {
        let mut topo = TopoSort::new();
        for node in graph.nodes() {
            if let Some(target) = node.target {
                let sources = node.sources.iter().filter_map(|x| x.clone()).collect::<Vec<_>>();
                topo.add(target, sources);
            }
        }
        PuzzleSolver {
            nodes: graph.nodes().to_vec(),
            order: topo.sort(),
            index: 0
        }
    }

    pub(super) fn run_one(&mut self, solution: &mut PuzzleSolution) -> bool {
        if self.index >= self.order.len() { return false; }
        let node = &self.nodes[self.order[self.index]];
        self.index += 1;
        if !solution.is_solved(&node.target) {
            (node.callback)(solution);
        }
        true
    }
}

#[cfg(test)]
mod test {
    use crate::puzzle::{puzzle::Puzzle, piece::{PuzzlePiece, PuzzleValue, ClonablePuzzleValue, PuzzleCombination, ErasedPiece}, PuzzleBuilder, PuzzleDependency};

    use super::*;

    struct Setup {
        s1: PuzzleSolution,
        p1: PuzzlePiece<i32>,
        p2: PuzzleCombination<i32>,
        p3: PuzzleCombination<i32>,
    }

    fn cmp_deps(deps: &[PuzzleDependency],indexes: &[Option<usize>]) {
        for (dep,index) in deps.iter().zip(indexes.iter()) {
            assert_eq!(&dep.partial_resolve(),index);
        }
    }

    fn setup() -> Setup {
        let builder = PuzzleBuilder::new();
        let p1 : PuzzlePiece<i32> = builder.new_piece();
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
        let s1 = PuzzleSolution::new(&puzzle);
        Setup { s1, p1, p2, p3 }
    }

    #[test]
    fn solver_smoke() {
        let mut setup = setup();
        let graph = setup.s1.graph();
        let mut solver = PuzzleSolver::new(&setup.s1,&graph);
        drop(graph);
        setup.p1.set_answer(&mut setup.s1,1);
        assert!(solver.run_one(&mut setup.s1));
        assert!(solver.run_one(&mut setup.s1));
        assert!(solver.run_one(&mut setup.s1));
        assert!(!solver.run_one(&mut setup.s1));
        assert!(setup.s1.all_solved());
    }

    #[test]
    fn solver_solvable() {
        let builder = PuzzleBuilder::new();
        let p1 : PuzzlePiece<i32> = builder.new_piece();
        let p3 = builder.new_combination(&[p1.dependency()], move |_| Some(0));
        let p2 = builder.new_combination(&[p1.dependency(),p3.dependency()], move |_| Some(0));
        let puzzle = Puzzle::new(builder);
        let mut s1 = PuzzleSolution::new(&puzzle);
        p1.set_answer(&mut s1,1);
        assert!(s1.solve());
    }

    #[test]
    fn graph_smoke() {
        let setup = setup();
        let graph = setup.s1.graph();
        assert_eq!(3,graph.nodes().len());
        cmp_deps(&[setup.p2.dependency()],&[graph.nodes()[1].target]);
        cmp_deps(&vec![setup.p1.dependency()],&graph.nodes()[1].sources);
        cmp_deps(&[setup.p3.dependency()],&[graph.nodes()[2].target]);
        cmp_deps(&vec![setup.p1.dependency(),setup.p2.dependency()],&graph.nodes()[2].sources);
    }
}
