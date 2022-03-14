use std::{sync::Arc, collections::HashMap};
use std::collections::HashSet;
use peregrine_toolkit::puzzle::{PuzzleSolution, Puzzle};

use super::{core::{ Patina, Pen, Plotter }, imageshape::ImageShape, rectangleshape::RectangleShape, textshape::TextShape, wiggleshape::WiggleShape};
use crate::allotment::boxes::root::PlayingField2;
use crate::allotment::core::allotmentmetadata2::AllotmentMetadataReport2;
use crate::{FloatingCarriageShapeList, ShapeRequest, ShapeRequestGroup, PlayingField};
use crate::{AllotmentMetadataStore, Assets, DataMessage, Shape, CarriageUniverse, AllotmentRequest, CarriageExtent, SpaceBaseArea, reactive::Observable, SpaceBase, allotment::{style::{pendingleaf::PendingLeaf, allotmentname::AllotmentName}, core::carriageuniverse2::{CarriageUniverse2, CarriageUniverseBuilder}, stylespec::{stylegroup::AllotmentStyleGroup, styletreebuilder::StyleTreeBuilder, styletree::StyleTree}}, EachOrEvery };

#[derive(Clone)]
enum PendingShape {
    Rectangle(SpaceBaseArea<f64,PendingLeaf>, Patina, Option<SpaceBaseArea<Observable<'static,f64>,()>>),
    Image(SpaceBase<f64,PendingLeaf>, EachOrEvery<String>),
    Text(SpaceBase<f64,PendingLeaf>, Pen,  EachOrEvery<String>),
    Wiggle(f64,f64, Plotter, Vec<Option<f64>>, PendingLeaf)
}

impl PendingShape {
    fn into_shape(&self, style: &AllotmentStyleGroup) -> Result<Vec<Shape<PendingLeaf>>,DataMessage> {
        match self {
            PendingShape::Rectangle(area,patina,wobble) => {
                let mut out = vec![];
                let depth = area.top_left().allotments().map(|leaf| {
                    style.get_pending_leaf(leaf).leaf.depth
                });
                for (coord_system,filter) in area.demerge_by_allotment(|leaf| {
                    &style.get_pending_leaf(leaf).leaf.top_style.coord_system
                }) {
                    let this_area = area.filter(&filter);
                    let this_depth = depth.filter(&filter);
                    let this_patina = patina.filter(&filter);
                    out.push(RectangleShape::new2(this_area,coord_system,&this_depth,this_patina,wobble.clone())?);
                }
                Ok(out)
            },
            PendingShape::Image(position,name) => {
                let mut out = vec![];
                let depth = position.allotments().map(|leaf| {
                    style.get_pending_leaf(leaf).leaf.depth
                });
                for (coord_system,filter) in position.demerge_by_allotment(|leaf| {
                    &style.get_pending_leaf(leaf).leaf.top_style.coord_system
                }) {
                    let this_position = position.filter(&filter);
                    let this_depth = depth.filter(&filter);
                    let this_name = name.filter(&filter);
                    out.push(ImageShape::new2(this_position,coord_system,&this_depth,this_name)?);
                }
                Ok(out)
            },
            PendingShape::Text(position,pen,text) => {
                let mut out = vec![];
                let depth = position.allotments().map(|leaf| {
                    style.get_pending_leaf(leaf).leaf.depth
                });
                for (coord_system,filter) in position.demerge_by_allotment(|leaf| {
                    &style.get_pending_leaf(leaf).leaf.top_style.coord_system
                }) {
                    let this_position = position.filter(&filter);
                    let this_depth = depth.filter(&filter);
                    let this_pen = pen.filter(&filter);
                    let this_text = text.filter(&filter);
                    out.push(TextShape::new2(this_position,coord_system,&this_depth,this_pen,this_text)?);
                }
                Ok(out)
            },
            // XXX don't copy values, use Arc
            PendingShape::Wiggle(min,max,plotter,values,leaf) => {
                let depth = style.get_pending_leaf(&leaf).leaf.depth;
                let coord_system = &style.get_pending_leaf(&leaf).leaf.top_style.coord_system;
                Ok(vec![WiggleShape::new2((*min,*max),values.clone(),depth,plotter.clone(),&leaf,coord_system)?])
            }
        }
    }
}

pub struct CarriageShapeListBuilder2 {
    shapes: Vec<PendingShape>,
    leafs: Vec<PendingLeaf>,
    carriage_universe: CarriageUniverseBuilder,
    style: StyleTreeBuilder
}

impl CarriageShapeListBuilder2 {
    pub fn new() -> CarriageShapeListBuilder2 {
        CarriageShapeListBuilder2 {
            shapes: vec![],
            leafs: vec![],
            carriage_universe: CarriageUniverseBuilder::new(),
            style: StyleTreeBuilder::new()
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
    
    pub fn add_rectangle(&mut self, area: SpaceBaseArea<f64,PendingLeaf>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<(),DataMessage> {
        self.shapes.push(PendingShape::Rectangle(area,patina,wobble));
        Ok(())
    }

    pub fn add_text(&mut self, position: SpaceBase<f64,PendingLeaf>, pen: Pen, text: EachOrEvery<String>) -> Result<(),DataMessage> {
        self.shapes.push(PendingShape::Text(position,pen,text));
        Ok(())
    }

    pub fn add_image(&mut self, position: SpaceBase<f64,PendingLeaf>, images: EachOrEvery<String>) -> Result<(),DataMessage> {
        self.shapes.push(PendingShape::Image(position,images));
        Ok(())
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: PendingLeaf) -> Result<(),DataMessage> {
        self.shapes.push(PendingShape::Wiggle(min,max,plotter,values,allotment));
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
        let mut shapes = vec![];
        for shape in &input.shapes {
            let mut these_shapes = shape.into_shape(&style)?;
            shapes.append(&mut these_shapes);
        }
        Ok(CarriageShapeListRaw {
            shapes: Arc::new(shapes),
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
            shapes: Arc::new(self.shapes.iter().map(|shape| shape.filter_by_minmax(min_value,max_value)).collect()),
            carriage_universe: self.carriage_universe.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CarriageShapeList2 {
    carriage_universe: Arc<CarriageUniverse2>
}

impl CarriageShapeList2 {
    pub fn empty() -> CarriageShapeList2 {
        CarriageShapeList2 {
            carriage_universe: Arc::new(CarriageUniverse2::new(&mut CarriageUniverseBuilder::new(),&vec![],None)),
        }
    }

    pub fn new(input: CarriageShapeListRaw, extent: Option<&ShapeRequestGroup>) -> Result<CarriageShapeList2,DataMessage> {
        let carriage_universe = CarriageUniverse2::new(&input.carriage_universe,&input.shapes,extent);
        Ok(CarriageShapeList2 {
            carriage_universe: Arc::new(carriage_universe),
        })
    }

    pub fn get(&self, solution: &PuzzleSolution) -> Vec<Shape<()>> { self.carriage_universe.get(solution) }
    pub fn get_metadata(&self, solution: &PuzzleSolution) -> AllotmentMetadataReport2 { self.carriage_universe.get_metadata(solution) }
    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField2 { self.carriage_universe.playing_field(solution) }    
    pub fn puzzle(&self) -> &Puzzle { self.carriage_universe.puzzle() }
}
