use std::sync::Arc;
use std::collections::HashSet;
use peregrine_toolkit::puzzle::{PuzzleSolution, Puzzle};

use super::{core::{ Patina, Pen, Plotter }, imageshape::ImageShape, rectangleshape::RectangleShape, textshape::TextShape, wiggleshape::WiggleShape};
use crate::{AllotmentMetadataStore, Assets, DataMessage, EachOrEvery, Shape, CarriageUniverse, AllotmentRequest, CarriageExtent, SpaceBaseArea, reactive::Observable, SpaceBase, allotment::{style::pendingleaf::PendingLeaf, core::carriageuniverse2::{CarriageUniverse2, CarriageUniverseBuilder}} };

pub struct CarriageShapeListBuilder2 {
    shapes: Vec<Shape<PendingLeaf>>,
    allotments: HashSet<AllotmentRequest>,
    assets: Assets,
    carriage_universe: CarriageUniverseBuilder
}

impl CarriageShapeListBuilder2 {
    pub fn new(allotment_metadata: &AllotmentMetadataStore, assets: &Assets) -> CarriageShapeListBuilder2 {
        CarriageShapeListBuilder2 {
            shapes: vec![],
            assets: assets.clone(),
            allotments: HashSet::new(),
            carriage_universe: CarriageUniverseBuilder::new()
        }
    }

    /*
    pub fn carriage_universe(&self) -> &CarriageUniverseBuilder { &self.carriage_universe }

    fn push(&mut self, shape: Shape<PendingLeaf>) {
        if !shape.is_empty() && !shape.common().coord_system().is_dustbin() {
            shape.register_space(&self.assets);
            self.shapes.push(shape);
        }
    }

    fn extend(&mut self, mut shapes: Vec<Shape<PendingLeaf>>) {
        for shape in shapes.drain(..) {
            self.push(shape);
        }
    }

    pub fn len(&self) -> usize { self.shapes.len() }

    pub fn vec_len(&self) -> usize {
        let mut out = 0;
        for shape in &self.shapes {
            out += shape.len();
        }
        out
    }

    pub fn use_allotment(&mut self, allotment: &AllotmentRequest) {
        if !allotment.is_dustbin() {
            self.allotments.insert(allotment.clone());
        }
    }
    
    pub fn add_rectangle(&mut self, area: SpaceBaseArea<f64,PendingLeaf>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<(),DataMessage> {
        let depth = area.top_left().allotments().map(|a| a.depth());
        self.extend(RectangleShape::<PendingLeaf>::new(area,depth,patina,wobble)?);
        Ok(())
    }

    pub fn add_text(&mut self, position: SpaceBase<f64,PendingLeaf>, pen: Pen, text: EachOrEvery<String>) -> Result<(),DataMessage> {
        let depth = position.allotments().map(|a| a.depth());
        self.extend(TextShape::<PendingLeaf>::new(position,depth,pen,text)?);
        Ok(())
    }

    pub fn add_image(&mut self, position: SpaceBase<f64,PendingLeaf>, images: EachOrEvery<String>) -> Result<(),DataMessage> {
        let depth = position.allotments().map(|a| a.depth());
        self.extend(ImageShape::<PendingLeaf>::new(position,depth,images)?);
        Ok(())
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: AllotmentRequest) -> Result<(),DataMessage> {
        let depth = EachOrEvery::every(allotment.depth());
        self.extend(WiggleShape::<PendingLeaf>::new((min,max),values,depth,plotter,allotment.clone())?);
        Ok(())
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> CarriageShapeListBuilder {
        let mut shapes = vec![];
        for shape in self.shapes.iter() {
            shapes.push(shape.filter_by_minmax(min_value,max_value));
        }
        CarriageShapeListBuilder { shapes, allotments: self.allotments.clone(), carriage_universe: self.carriage_universe.clone(), assets: self.assets.clone() }
    }

    pub fn append(&mut self, more: &CarriageShapeListBuilder) {
        self.shapes.extend(more.shapes.iter().cloned());
        self.allotments = self.allotments.union(&more.allotments).cloned().collect();
        self.carriage_universe.union(&more.carriage_universe);
    }
}

#[derive(Clone)]
pub struct FloatingCarriageShapeList2 {
    shapes: Arc<Vec<Shape<PendingLeaf>>>,
    carriage_universe: CarriageUniverse,
    extent: Option<CarriageExtent>
}

impl FloatingCarriageShapeList2 {
    pub fn empty() -> FloatingCarriageShapeList2 {
        FloatingCarriageShapeList2 {
            shapes: Arc::new(vec![]),
            carriage_universe: CarriageUniverse::new(&AllotmentMetadataStore::new()),
            extent: None
        }
    }

    pub fn len(&self) -> usize { self.shapes.len() }

    pub fn new(builder: CarriageShapeListBuilder2, extent: Option<&CarriageExtent>) -> Result<FloatingCarriageShapeList2,DataMessage> {
        Ok(FloatingCarriageShapeList2 {
            shapes: Arc::new(builder.shapes),
            carriage_universe: builder.carriage_universe,
            extent: extent.cloned()
        })
    }
}

#[derive(Clone)]
pub struct AnchoredCarriageShapeList2 {
    shapes: Arc<Vec<Shape<()>>>,
    carriage_universe: CarriageUniverse
}

impl AnchoredCarriageShapeList2 {
    pub fn empty() -> Result<AnchoredCarriageShapeList2,DataMessage> {
        AnchoredCarriageShapeList2::new(&FloatingCarriageShapeList2::empty())
    }

    pub fn new(floating: &FloatingCarriageShapeList2) -> Result<AnchoredCarriageShapeList2,DataMessage> {
        let puzzle = Puzzle::new(floating.carriage_universe.puzzle());
        let mut solution = PuzzleSolution::new(&puzzle);
        /* allotments are assigned here */
        floating.carriage_universe.allot(floating.extent.as_ref());
        /* shapes mapped to allotments here */
        let mut shapes = floating.shapes.iter()
            .map(|s| s.clone().allot(|r| r.allotment()))
            .collect::<Result<Vec<_>,_>>()?;
        let shapes = shapes.drain(..).map(|s| s.transform(&solution)).collect();
        Ok(AnchoredCarriageShapeList2 {
            carriage_universe: floating.carriage_universe.clone(),
            shapes: Arc::new(shapes)
        })
    }

    pub fn carriage_universe(&self) -> &CarriageUniverse { &self.carriage_universe}
    pub fn shapes(&self) -> &Arc<Vec<Shape<()>>> { &self.shapes }
    */
}
