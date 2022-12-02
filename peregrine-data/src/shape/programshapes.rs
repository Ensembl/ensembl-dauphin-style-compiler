use std::{collections::{HashMap, HashSet}};
use peregrine_toolkit::{debug_log, eachorevery::EachOrEvery};
use super::{core::{ Patina, Pen, Plotter }, imageshape::ImageShape, rectangleshape::RectangleShape, textshape::TextShape, wiggleshape::WiggleShape, emptyshape::EmptyShape, shape::UnplacedShape};
use crate::{LeafRequest, FloatingShapesContainer, allotment::core::leaflist::LeafList, BackendNamespace, LoadMode, PolygonShape};
use crate::{Assets, DataMessage, SpaceBaseArea, reactive::Observable, SpaceBase, allotment::{stylespec::{stylegroup::AllStylesForProgram, styletreebuilder::StyleTreeBuilder, styletree::StyleTree}}};

pub struct ProgramShapesBuilder {
    assets: Assets,
    shapes: Vec<UnplacedShape>,
    leafs: HashSet<LeafRequest>,
    leaf_list: LeafList,
    style: StyleTreeBuilder,
    mode: LoadMode
}

impl ProgramShapesBuilder {
    pub fn new(assets: &Assets, mode: &LoadMode) -> ProgramShapesBuilder {
        ProgramShapesBuilder {
            shapes: vec![],
            leafs: HashSet::new(),
            leaf_list: LeafList::new(),
            style: StyleTreeBuilder::new(),
            assets: assets.clone(),
            mode: mode.clone()
        }
    }

    pub fn use_allotment(&mut self, spec: &str) -> &LeafRequest {
        let leaf = self.leaf_list.pending_leaf(spec);
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
        self.push_shape(RectangleShape::new(area,patina,wobble)?);
        Ok(())
    }

    pub fn add_polygon(&mut self, position: SpaceBase<f64,LeafRequest>, radius: EachOrEvery<f64>, points: usize, angle: f32, patina: Patina, wobble: Option<SpaceBase<Observable<'static,f64>,()>>) -> Result<(),DataMessage> {
        self.push_shape(PolygonShape::new(position,radius,points,angle,patina,wobble)?);
        Ok(())
    }

    pub fn add_text(&mut self, position: SpaceBase<f64,LeafRequest>, pen: Pen, text: EachOrEvery<String>) -> Result<(),DataMessage> {
        self.push_shape(TextShape::new(position,pen,text)?);
        Ok(())
    }

    pub fn add_running_text(&mut self, area: SpaceBaseArea<f64,LeafRequest>, pen: Pen, text: EachOrEvery<String>) -> Result<(),DataMessage> {
        let bottom_right = area.bottom_right().map_allotments(|_| ());
        self.push_shape(TextShape::new_running(area.top_left().clone(),bottom_right,pen,text)?);
        Ok(())
    }

    pub fn add_image(&mut self, channel: &BackendNamespace, position: SpaceBase<f64,LeafRequest>, images: EachOrEvery<String>) -> Result<(),DataMessage> {
        self.push_shape(ImageShape::new(position,channel,images)?);
        Ok(())
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: LeafRequest) -> Result<(),DataMessage> {
        self.push_shape(WiggleShape::new2((min,max),values,plotter,allotment)?);
        Ok(())
    }

    pub fn to_abstract_shapes_container(self) -> FloatingShapesContainer {
        let style = AllStylesForProgram::new(StyleTree::new(self.style));
        if self.leafs.len() > 1000 {
            debug_log!("many leafs! {}",self.leafs.len());
        }
        for leaf in self.leafs {
            leaf.set_style(&style);
        }
        FloatingShapesContainer::build(self.shapes,self.leaf_list,&self.mode)
    }
}
