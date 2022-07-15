use std::{collections::{HashMap, HashSet}};

use peregrine_toolkit::{timer_start, timer_end, log, eachorevery::EachOrEvery};

use super::{core::{ Patina, Pen, Plotter }, imageshape::ImageShape, rectangleshape::RectangleShape, textshape::TextShape, wiggleshape::WiggleShape, emptyshape::EmptyShape, shape::UnplacedShape};
use crate::{LeafRequest, AbstractShapesContainer, allotment::core::leaflist::LeafList};
use crate::{Assets, DataMessage, SpaceBaseArea, reactive::Observable, SpaceBase, allotment::{stylespec::{stylegroup::AllotmentStyleGroup, styletreebuilder::StyleTreeBuilder, styletree::StyleTree}}};

pub struct ProgramShapesBuilder {
    assets: Assets,
    shapes: Vec<UnplacedShape>,
    leafs: HashSet<LeafRequest>,
    carriage_universe: LeafList,
    style: StyleTreeBuilder
}

impl ProgramShapesBuilder {
    pub fn new(assets: &Assets) -> ProgramShapesBuilder {
        ProgramShapesBuilder {
            shapes: vec![],
            leafs: HashSet::new(),
            carriage_universe: LeafList::new(),
            style: StyleTreeBuilder::new(),
            assets: assets.clone()
        }
    }

    pub fn use_allotment(&mut self, spec: &str) -> &LeafRequest {
        let leaf = self.carriage_universe.pending_leaf(spec);
        self.leafs.insert(leaf.clone());
        leaf
    }

    pub fn add_style(&mut self, spec: &str, props: HashMap<String,String>) {
        self.style.add(spec,props);
    }

    pub fn len(&self) -> usize { self.shapes.len() }

    fn push_shape(&mut self, shape: UnplacedShape) {
        shape.register_space(&self.assets);
        self.shapes.push(shape);
    }

    pub fn add_empty(&mut self, area: SpaceBaseArea<f64,LeafRequest>) -> Result<(),DataMessage> {
        self.push_shape(EmptyShape::new(area)?);
        Ok(())
    }

    pub fn add_rectangle(&mut self, area: SpaceBaseArea<f64,LeafRequest>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<(),DataMessage> {
        self.push_shape(RectangleShape::new2(area,patina,wobble)?);
        Ok(())
    }

    pub fn add_text(&mut self, position: SpaceBase<f64,LeafRequest>, pen: Pen, text: EachOrEvery<String>) -> Result<(),DataMessage> {
        self.push_shape(TextShape::new(position,pen,text)?);
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

    pub fn to_abstract_shapes_container(self) -> AbstractShapesContainer {
        let style = AllotmentStyleGroup::new(StyleTree::new(self.style));
        if self.leafs.len() > 1000 {
            log!("many leafs! {}",self.leafs.len());
        }
        for leaf in self.leafs {
            leaf.set_style(&style);
        }
        AbstractShapesContainer::build(self.shapes,self.carriage_universe)
    }
}
