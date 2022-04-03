use std::{collections::HashMap};

use super::{core::{ Patina, Pen, Plotter }, imageshape::ImageShape, rectangleshape::RectangleShape, textshape::TextShape, wiggleshape::WiggleShape};
use crate::{Shape, LeafRequest, CarriageShapesBuilder, allotment::core::leaflist::LeafList};
use crate::{Assets, DataMessage, SpaceBaseArea, reactive::Observable, SpaceBase, allotment::{stylespec::{stylegroup::AllotmentStyleGroup, styletreebuilder::StyleTreeBuilder, styletree::StyleTree}}, EachOrEvery };

pub struct ProgramShapesBuilder {
    assets: Assets,
    shapes: Vec<Shape<LeafRequest>>,
    leafs: Vec<LeafRequest>,
    carriage_universe: LeafList,
    style: StyleTreeBuilder
}

impl ProgramShapesBuilder {
    pub fn new(assets: &Assets) -> ProgramShapesBuilder {
        ProgramShapesBuilder {
            shapes: vec![],
            leafs: vec![],
            carriage_universe: LeafList::new(),
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

    /* Only to be called during adding to CarriageShapes */
    pub(super) fn to_carriage_shapes_builder(self) -> CarriageShapesBuilder {
        let style = AllotmentStyleGroup::new(StyleTree::new(self.style));
        for leaf in self.leafs {
            leaf.set_style(&style);
        }
        CarriageShapesBuilder::build(self.shapes,self.carriage_universe)
    }
}
