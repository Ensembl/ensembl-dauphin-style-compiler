use std::{collections::HashMap, sync::Arc};

use peregrine_toolkit::puzzle::{PuzzleBuilder, PuzzleSolution, Puzzle};

use crate::{allotment::{style::{pendingleaf::PendingLeaf, allotmentname::AllotmentName, holder::ContainerHolder, stylebuilder::make_transformable }, stylespec::stylegroup::AllotmentStyleGroup, boxes::{root::{Root, PlayingField2}, boxtraits::Transformable}}, Pen, CarriageExtent, Shape, ShapeRequest, ShapeRequestGroup, EachOrEvery, PlayingField};

use super::{arbitrator::BpPxConverter, allotmentmetadata2::{AllotmentMetadataReport2, AllotmentMetadata2, AllotmentMetadata2Builder}};

pub struct CarriageUniverseBuilder {
    leafs: HashMap<String,PendingLeaf>
}

impl CarriageUniverseBuilder {
    pub fn new() -> CarriageUniverseBuilder {
        CarriageUniverseBuilder {
            leafs: HashMap::new(),
        }
    }

    pub fn pending_leaf(&mut self, spec: &str) -> &mut PendingLeaf {
        if !self.leafs.contains_key(spec) {
            self.leafs.insert(spec.to_string(),PendingLeaf::new(&AllotmentName::new(spec)));
        }
        self.leafs.get_mut(spec).unwrap()
    }

    pub fn union(&self, other: &CarriageUniverseBuilder) -> CarriageUniverseBuilder {
        let mut leafs = self.leafs.clone();
        leafs.extend(other.leafs.iter().map(|(k,v)| (k.clone(),v.clone())));
        CarriageUniverseBuilder {
            leafs
        }
    }

    fn make_transformable(&self, extent: Option<&ShapeRequestGroup>) -> (Puzzle,AllotmentMetadata2Builder,Root) {
        let mut metadata = AllotmentMetadata2Builder::new();
        let mut builder = PuzzleBuilder::new();
        let converter = Arc::new(BpPxConverter::new(extent));
        let root = Root::new(&mut builder);
        make_transformable(&builder,&converter,&ContainerHolder::Root(root.clone()),&mut self.leafs.values(),&mut metadata);
        (Puzzle::new(builder),metadata,root)
    }
}

#[derive(Clone)]
pub struct CarriageUniverse2 {
    shapes: Arc<Vec<Shape<Arc<dyn Transformable>>>>,
    metadata: AllotmentMetadata2,
    puzzle: Puzzle,
    root: Root
}

impl CarriageUniverse2 {
    pub fn new(builder: &CarriageUniverseBuilder, shapes: &[Shape<PendingLeaf>], extent: Option<&ShapeRequestGroup>) -> CarriageUniverse2 {
        let (puzzle,metadata,root) = builder.make_transformable(extent);
        let shapes = shapes.iter().map(|x| 
            x.map_new_allotment(|x| x.transformable().cloned())
        ).collect::<Vec<_>>();
        CarriageUniverse2 {
            shapes: Arc::new(shapes),
            metadata: AllotmentMetadata2::new(&metadata),
            puzzle, root
        }
    }

    pub fn puzzle(&self) -> &Puzzle { &self.puzzle }

    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField2 {
        self.root.playing_field(solution)
    }

    pub fn get(&self, solution: &PuzzleSolution) -> Vec<Shape<()>> {
        let mut out = vec![];
        for input in self.shapes.iter() {
            out.append(&mut input.map_new_allotment(|x| x.make(solution)).make(solution));
        }
        out
    }

    pub fn get_metadata(&self, solution: &PuzzleSolution) -> AllotmentMetadataReport2 {
        self.metadata.get(solution)
    }
}
