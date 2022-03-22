use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::{puzzle::{PuzzleBuilder, PuzzleSolution, Puzzle}, error, lock, log};

use crate::{allotment::{style::{pendingleaf::{PendingLeaf, PendingLeafSource, PendingLeafMap}, allotmentname::AllotmentName, holder::ContainerHolder, stylebuilder::make_transformable, style::LeafCommonStyle }, stylespec::stylegroup::AllotmentStyleGroup, boxes::{root::{Root, PlayingField2}, boxtraits::Transformable}}, Pen, CarriageExtent, ShapeRequest, ShapeRequestGroup, EachOrEvery, PlayingField, Shape, DataMessage};

use super::{arbitrator::BpPxConverter, allotmentmetadata2::{AllotmentMetadataReport2, AllotmentMetadata2, AllotmentMetadata2Builder}, aligner::Aligner};

pub struct CarriageUniverseBuilder {
    source: PendingLeafSource,
    leafs: HashMap<String,PendingLeaf>
}

impl CarriageUniverseBuilder {
    pub fn new() -> CarriageUniverseBuilder {
        CarriageUniverseBuilder {
            source: PendingLeafSource::new(),
            leafs: HashMap::new()
        }
    }

    pub fn pending_leaf(&mut self, spec: &str) -> &mut PendingLeaf {
        if !self.leafs.contains_key(spec) {
            self.leafs.insert(spec.to_string(),PendingLeaf::new(&mut self.source,&AllotmentName::new(spec)));
        }
        self.leafs.get_mut(spec).unwrap()
    }

    pub fn union(&self, other: &CarriageUniverseBuilder) -> CarriageUniverseBuilder {
        let mut new_source = self.source.clone();
        let mut leafs = self.leafs.clone();
        leafs.extend(other.leafs.iter().map(|(k,v)| {
            let mut v = v.clone();
            v.new_source(&mut new_source);
            (k.clone(),v)
        }));
        CarriageUniverseBuilder {
            source: new_source,
            leafs
        }
    }

    fn make_transformable(&self, extent: Option<&ShapeRequestGroup>) -> Result<(Puzzle,AllotmentMetadata2Builder,Root,PendingLeafMap),DataMessage> {
        let mut plm = PendingLeafMap::new(&self.source);
        let mut metadata = AllotmentMetadata2Builder::new();
        let mut builder = PuzzleBuilder::new();
        let converter = Arc::new(BpPxConverter::new(extent));
        let root = Root::new(&mut builder);
        let aligner = Aligner::new(&root);
        make_transformable(&builder,&mut plm,&converter,&ContainerHolder::Root(root.clone()),&mut self.leafs.values(),&mut metadata,&aligner)?;
        Ok((Puzzle::new(builder),metadata,root,plm))
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
    pub fn new(builder: &CarriageUniverseBuilder, shapes: &[Shape<PendingLeaf>], extent: Option<&ShapeRequestGroup>) -> Result<CarriageUniverse2,DataMessage> {
        let (puzzle,metadata,root,plm) = builder.make_transformable(extent)?;
        let shapes = shapes.iter().map(|x| 
            x.map_new_allotment(|x| x.transformable(&plm).cloned())
        ).collect::<Vec<_>>();
        Ok(CarriageUniverse2 {
            shapes: Arc::new(shapes),
            metadata: AllotmentMetadata2::new(&metadata),
            puzzle, root
        })
    }

    pub fn puzzle(&self) -> &Puzzle { &self.puzzle }

    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField2 {
        self.root.playing_field(solution)
    }

    pub fn get(&self, solution: &PuzzleSolution) -> Vec<Shape<LeafCommonStyle>> {
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
