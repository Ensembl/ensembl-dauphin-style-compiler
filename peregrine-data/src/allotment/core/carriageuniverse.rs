use std::{collections::HashMap, sync::{Arc}};
use peregrine_toolkit::{puzzle::{PuzzleBuilder, PuzzleSolution, Puzzle}};

use crate::{allotment::{style::{allotmentname::{AllotmentName, new_efficient_allotmentname_hashmap, BuildPassThroughHasher}, holder::ContainerHolder, stylebuilder::make_transformable, style::LeafCommonStyle }, boxes::{root::{Root}, boxtraits::Transformable}, util::bppxconverter::BpPxConverter}, ShapeRequestGroup, Shape, DataMessage, LeafRequest};

use super::{allotmentmetadata::{AllotmentMetadataReport, AllotmentMetadata, AllotmentMetadataBuilder}, aligner::Aligner, playingfield::PlayingField, leafrequest::LeafRequestMap};

pub struct CarriageUniverseBuilder {
    leafs: HashMap<AllotmentName,LeafRequest,BuildPassThroughHasher>
}

impl CarriageUniverseBuilder {
    pub fn new() -> CarriageUniverseBuilder {
        CarriageUniverseBuilder {
            leafs: new_efficient_allotmentname_hashmap()
        }
    }

    pub fn pending_leaf(&mut self, spec: &str) -> &mut LeafRequest {
        let name = AllotmentName::new(spec);
        if !self.leafs.contains_key(&name) {
            self.leafs.insert(name.clone(),LeafRequest::new(&AllotmentName::new(spec)));
        }
        self.leafs.get_mut(&name).unwrap()
    }

    pub fn union(&self, other: &CarriageUniverseBuilder) -> CarriageUniverseBuilder {
        let mut leafs = self.leafs.clone();
        leafs.extend(&mut other.leafs.iter().map(|(k,v)| (k.clone(),v.clone())));
        CarriageUniverseBuilder {
            leafs
        }
    }

    fn make_transformable(&self, extent: Option<&ShapeRequestGroup>) -> Result<(Puzzle,AllotmentMetadataBuilder,Root,LeafRequestMap),DataMessage> {
        let mut plm = LeafRequestMap::new();
        let mut metadata = AllotmentMetadataBuilder::new();
        let mut builder = PuzzleBuilder::new();
        let converter = Arc::new(BpPxConverter::new(extent));
        let root = Root::new(&mut builder);
        let aligner = Aligner::new(&root);
        make_transformable(&builder,&mut plm,&converter,&ContainerHolder::Root(root.clone()),&mut self.leafs.values(),&mut metadata,&aligner)?;
        Ok((Puzzle::new(builder),metadata,root,plm))
    }
}

#[derive(Clone)]
pub struct CarriageUniverse {
    shapes: Arc<Vec<Shape<Arc<dyn Transformable>>>>,
    metadata: AllotmentMetadata,
    puzzle: Puzzle,
    root: Root
}

impl CarriageUniverse {
    pub fn new(builder: &CarriageUniverseBuilder, shapes: &[Shape<LeafRequest>], extent: Option<&ShapeRequestGroup>) -> Result<CarriageUniverse,DataMessage> {
        let (puzzle,metadata,root,plm) = builder.make_transformable(extent)?;
        let shapes = shapes.iter().map(|x| 
            x.map_new_allotment(|x| x.transformable(&plm).cloned())
        ).collect::<Vec<_>>();
        Ok(CarriageUniverse {
            shapes: Arc::new(shapes),
            metadata: AllotmentMetadata::new(&metadata),
            puzzle, root
        })
    }

    pub fn puzzle(&self) -> &Puzzle { &self.puzzle }

    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField {
        self.root.playing_field(solution)
    }

    pub fn get(&self, solution: &PuzzleSolution) -> Vec<Shape<LeafCommonStyle>> {
        let mut out = vec![];
        for input in self.shapes.iter() {
            out.append(&mut input.map_new_allotment(|x| x.make(solution)).make(solution));
        }
        out
    }

    pub fn get_metadata(&self, solution: &PuzzleSolution) -> AllotmentMetadataReport {
        self.metadata.get(solution)
    }
}
