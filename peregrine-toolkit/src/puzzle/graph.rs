use std::{sync::Arc};

#[cfg(test)]
use std::fmt;

use super::{puzzle::{PuzzleDependency}, PuzzleBuilder, PuzzleSolution, solver::PuzzleSolverNode};

#[derive(Clone)]
pub(super) struct PuzzleGraphNode {
    target: PuzzleDependency,
    sources: Vec<PuzzleDependency>,
    callback: Arc<dyn Fn(&mut PuzzleSolution)>
}

impl PuzzleGraphNode {
    pub(super) fn to_puzzle_solver_node(&self, builder: &PuzzleBuilder) -> PuzzleSolverNode {
        PuzzleSolverNode::new(
            self.target.resolve(builder),
            self.sources.iter().map(|x| x.resolve(builder)).collect(),
            self.callback.clone()
        )
    }
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

    pub(super) fn add_known(&mut self, target: &PuzzleDependency) {
        self.nodes.push(PuzzleGraphNode {
            target: target.clone(),
            sources: vec![],
            callback: Arc::new(|_| {})
        })
    }

    pub(super) fn add_solver(&mut self, target: &PuzzleDependency, sources: &[PuzzleDependency], callback: Arc<dyn Fn(&mut PuzzleSolution)>) {
        self.nodes.push(PuzzleGraphNode {
            target: target.clone(),
            sources: sources.to_vec(),
            callback
        })
    }
}

#[derive(Clone)]
pub(super) struct PuzzleGraphReady {
    nodes: Arc<Vec<PuzzleSolverNode>>
}

impl PuzzleGraphReady {
    pub(super) fn new(builder: &PuzzleBuilder, graph: &PuzzleGraph) -> PuzzleGraphReady {
        PuzzleGraphReady {
            nodes: Arc::new(graph.nodes.iter().map(|n| n.to_puzzle_solver_node(builder)).collect())
        }
    }

    pub(super) fn nodes(&self) -> &[PuzzleSolverNode] { &self.nodes }
}
