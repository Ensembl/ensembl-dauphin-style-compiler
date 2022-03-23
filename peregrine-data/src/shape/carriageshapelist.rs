use std::{sync::Arc, collections::HashMap};
use peregrine_toolkit::puzzle::{PuzzleSolution, Puzzle};

use super::{core::{ Patina, Pen, Plotter }, imageshape::ImageShape, rectangleshape::RectangleShape, textshape::TextShape, wiggleshape::WiggleShape};
use crate::allotment::core::allotmentmetadata::AllotmentMetadataReport;
use crate::allotment::style::style::LeafCommonStyle;
use crate::{ShapeRequestGroup, Shape, PlayingField, LeafRequest};
use crate::{Assets, DataMessage, SpaceBaseArea, reactive::Observable, SpaceBase, allotment::{core::carriageuniverse::{CarriageUniverse, CarriageUniverseBuilder}, stylespec::{stylegroup::AllotmentStyleGroup, styletreebuilder::StyleTreeBuilder, styletree::StyleTree}}, EachOrEvery };

pub struct CarriageShapeListBuilder {
    assets: Assets,
    shapes: Vec<Shape<LeafRequest>>,
    leafs: Vec<LeafRequest>,
    carriage_universe: CarriageUniverseBuilder,
    style: StyleTreeBuilder
}

impl CarriageShapeListBuilder {
    pub fn new(assets: &Assets) -> CarriageShapeListBuilder {
        CarriageShapeListBuilder {
            shapes: vec![],
            leafs: vec![],
            carriage_universe: CarriageUniverseBuilder::new(),
            style: StyleTreeBuilder::new(),
            assets: assets.clone()
        }
    }

    pub fn use_allotment(&mut self, spec: &str) -> &LeafRequest {
        let leaf = self.carriage_universe.pending_leaf(spec);
        self.leafs.push(leaf.clone());
        leaf
    }

    pub fn add_style(&mut self, spec: &str, props: HashMap<String,String>) {
        self.style.add(spec,props);
    }

    pub fn len(&self) -> usize { self.shapes.len() }

    fn push_shape(&mut self, shape: Shape<LeafRequest>) {
        shape.register_space(&self.assets);
        self.shapes.push(shape);
    }

    pub fn add_rectangle(&mut self, area: SpaceBaseArea<f64,LeafRequest>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<(),DataMessage> {
        self.push_shape(RectangleShape::new2(area,patina,wobble)?);
        Ok(())
    }

    pub fn add_text(&mut self, position: SpaceBase<f64,LeafRequest>, pen: Pen, text: EachOrEvery<String>) -> Result<(),DataMessage> {
        self.push_shape(TextShape::new2(position,pen,text)?);
        Ok(())
    }

    pub fn add_image(&mut self, position: SpaceBase<f64,LeafRequest>, images: EachOrEvery<String>) -> Result<(),DataMessage> {
        self.push_shape(ImageShape::new2(position,images)?);
        Ok(())
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: LeafRequest) -> Result<(),DataMessage> {
        self.push_shape(WiggleShape::new2((min,max),values,plotter,allotment)?);
        Ok(())
    }

    pub fn append(&mut self, more: &CarriageShapeListBuilder) {
        self.shapes.extend(more.shapes.iter().cloned());
        self.carriage_universe.union(&more.carriage_universe);
    }
}

#[derive(Clone)]
pub struct CarriageShapeListRaw {
    shapes: Arc<Vec<Shape<LeafRequest>>>,
    carriage_universe: Arc<CarriageUniverseBuilder>
}

impl CarriageShapeListRaw {
    pub fn new(input: CarriageShapeListBuilder) -> Result<CarriageShapeListRaw,DataMessage> {
        let style = AllotmentStyleGroup::new(StyleTree::new(input.style));
        for leaf in input.leafs {
            leaf.set_style(&style);
        }
        Ok(CarriageShapeListRaw {
            shapes: Arc::new(input.shapes),
            carriage_universe: Arc::new(input.carriage_universe)
        })
    }

    pub fn empty() -> CarriageShapeListRaw {
        CarriageShapeListRaw {
            shapes: Arc::new(vec![]),
            carriage_universe: Arc::new(CarriageUniverseBuilder::new())
        }
    }

    pub fn union(&self, more: &CarriageShapeListRaw) -> CarriageShapeListRaw {
        let mut shapes = self.shapes.as_ref().to_vec();
        shapes.extend(more.shapes.iter().cloned());
        CarriageShapeListRaw {
            shapes: Arc::new(shapes),
            carriage_universe: Arc::new(self.carriage_universe.union(&more.carriage_universe))
        }
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> CarriageShapeListRaw {
        CarriageShapeListRaw {
            shapes: Arc::new(self.shapes.iter().map(|shape| shape.base_filter(min_value,max_value)).collect()),
            carriage_universe: self.carriage_universe.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CarriageShapeList {
    carriage_universe: Arc<CarriageUniverse>
}

impl CarriageShapeList {
    pub fn empty() -> Result<CarriageShapeList,DataMessage> {
        Ok(CarriageShapeList {
            carriage_universe: Arc::new(CarriageUniverse::new(&mut CarriageUniverseBuilder::new(),&vec![],None)?),
        })
    }

    pub fn new(input: CarriageShapeListRaw, extent: Option<&ShapeRequestGroup>) -> Result<CarriageShapeList,DataMessage> {
        let carriage_universe = CarriageUniverse::new(&input.carriage_universe,&input.shapes,extent)?;
        Ok(CarriageShapeList {
            carriage_universe: Arc::new(carriage_universe),
        })
    }

    pub fn get(&self, solution: &PuzzleSolution) -> Vec<Shape<LeafCommonStyle>> { self.carriage_universe.get(solution) }
    pub fn get_metadata(&self, solution: &PuzzleSolution) -> AllotmentMetadataReport { self.carriage_universe.get_metadata(solution) }
    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField { self.carriage_universe.playing_field(solution) }    
    pub fn puzzle(&self) -> &Puzzle { self.carriage_universe.puzzle() }
}
