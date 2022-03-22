use std::{sync::Arc, collections::HashMap};
use std::collections::HashSet;
use peregrine_toolkit::puzzle::{PuzzleSolution, Puzzle};

use super::{core::{ Patina, Pen, Plotter }, imageshape::ImageShape, rectangleshape::RectangleShape, textshape::TextShape, wiggleshape::WiggleShape};
use crate::allotment::boxes::root::PlayingField2;
use crate::allotment::core::allotmentmetadata2::AllotmentMetadataReport2;
use crate::allotment::style::style::LeafCommonStyle;
use crate::{ShapeRequest, ShapeRequestGroup, Shape};
use crate::{Assets, DataMessage, CarriageExtent, SpaceBaseArea, reactive::Observable, SpaceBase, allotment::{style::{pendingleaf::PendingLeaf, allotmentname::AllotmentName}, core::carriageuniverse2::{CarriageUniverse2, CarriageUniverseBuilder}, stylespec::{stylegroup::AllotmentStyleGroup, styletreebuilder::StyleTreeBuilder, styletree::StyleTree}}, EachOrEvery };

pub struct CarriageShapeListBuilder2 {
    assets: Assets,
    shapes: Vec<Shape<PendingLeaf>>,
    leafs: Vec<PendingLeaf>,
    carriage_universe: CarriageUniverseBuilder,
    style: StyleTreeBuilder
}

impl CarriageShapeListBuilder2 {
    pub fn new(assets: &Assets) -> CarriageShapeListBuilder2 {
        CarriageShapeListBuilder2 {
            shapes: vec![],
            leafs: vec![],
            carriage_universe: CarriageUniverseBuilder::new(),
            style: StyleTreeBuilder::new(),
            assets: assets.clone()
        }
    }

    pub fn use_allotment(&mut self, spec: &str) -> &PendingLeaf {
        let leaf = self.carriage_universe.pending_leaf(spec);
        self.leafs.push(leaf.clone());
        leaf
    }

    pub fn add_style(&mut self, spec: &str, props: HashMap<String,String>) {
        self.style.add(spec,props);
    }

    pub fn len(&self) -> usize { self.shapes.len() }

    fn push_shape(&mut self, shape: Shape<PendingLeaf>) {
        shape.register_space(&self.assets);
        self.shapes.push(shape);
    }

    pub fn add_rectangle(&mut self, area: SpaceBaseArea<f64,PendingLeaf>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<(),DataMessage> {
        self.push_shape(RectangleShape::new2(area,patina,wobble)?);
        Ok(())
    }

    pub fn add_text(&mut self, position: SpaceBase<f64,PendingLeaf>, pen: Pen, text: EachOrEvery<String>) -> Result<(),DataMessage> {
        self.push_shape(TextShape::new2(position,pen,text)?);
        Ok(())
    }

    pub fn add_image(&mut self, position: SpaceBase<f64,PendingLeaf>, images: EachOrEvery<String>) -> Result<(),DataMessage> {
        self.push_shape(ImageShape::new2(position,images)?);
        Ok(())
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: PendingLeaf) -> Result<(),DataMessage> {
        self.push_shape(WiggleShape::new2((min,max),values,plotter,allotment)?);
        Ok(())
    }

    pub fn append(&mut self, more: &CarriageShapeListBuilder2) {
        self.shapes.extend(more.shapes.iter().cloned());
        self.carriage_universe.union(&more.carriage_universe);
    }
}

#[derive(Clone)]
pub struct CarriageShapeListRaw {
    shapes: Arc<Vec<Shape<PendingLeaf>>>,
    carriage_universe: Arc<CarriageUniverseBuilder>
}

impl CarriageShapeListRaw {
    pub fn new(input: CarriageShapeListBuilder2) -> Result<CarriageShapeListRaw,DataMessage> {
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
pub struct CarriageShapeList2 {
    carriage_universe: Arc<CarriageUniverse2>
}

impl CarriageShapeList2 {
    pub fn empty() -> Result<CarriageShapeList2,DataMessage> {
        Ok(CarriageShapeList2 {
            carriage_universe: Arc::new(CarriageUniverse2::new(&mut CarriageUniverseBuilder::new(),&vec![],None)?),
        })
    }

    pub fn new(input: CarriageShapeListRaw, extent: Option<&ShapeRequestGroup>) -> Result<CarriageShapeList2,DataMessage> {
        let carriage_universe = CarriageUniverse2::new(&input.carriage_universe,&input.shapes,extent)?;
        Ok(CarriageShapeList2 {
            carriage_universe: Arc::new(carriage_universe),
        })
    }

    pub fn get(&self, solution: &PuzzleSolution) -> Vec<Shape<LeafCommonStyle>> { self.carriage_universe.get(solution) }
    pub fn get_metadata(&self, solution: &PuzzleSolution) -> AllotmentMetadataReport2 { self.carriage_universe.get_metadata(solution) }
    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField2 { self.carriage_universe.playing_field(solution) }    
    pub fn puzzle(&self) -> &Puzzle { self.carriage_universe.puzzle() }
}
