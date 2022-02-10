use std::sync::Arc;
use std::collections::HashSet;
use super::{core::{ Patina, Pen, Plotter }, imageshape::ImageShape, rectangleshape::RectangleShape, textshape::TextShape, wiggleshape::WiggleShape};
use crate::{AllotmentMetadataStore, Assets, DataMessage, EachOrEvery, HoleySpaceBase, HoleySpaceBaseArea, Shape, Universe, AllotmentRequest, Scale, core::pixelsize::PixelSize, CarriageExtent, Allotment, allotment::core::allotmentrequest::GenericAllotmentRequestImpl };

pub struct ShapeListBuilder {
    shapes: Vec<Shape<AllotmentRequest>>,
    allotments: HashSet<AllotmentRequest>,
    assets: Assets,
    universe: Universe
}

impl ShapeListBuilder {
    pub fn new(allotment_metadata: &AllotmentMetadataStore, assets: &Assets) -> ShapeListBuilder {
        ShapeListBuilder {
            shapes: vec![],
            assets: assets.clone(),
            allotments: HashSet::new(),
            universe: Universe::new(allotment_metadata)
        }
    }

    pub fn universe(&self) -> &Universe { &self.universe }

    fn push(&mut self, shape: Shape<AllotmentRequest>) {
        if !shape.is_empty() && !shape.common().coord_system().is_dustbin() {
            shape.register_space(&self.assets);
            self.shapes.push(shape);
        }
    }

    fn extend(&mut self, mut shapes: Vec<Shape<AllotmentRequest>>) {
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
    
    pub fn add_rectangle(&mut self, area: HoleySpaceBaseArea<f64>, patina: Patina, allotments: EachOrEvery<AllotmentRequest>) -> Result<(),DataMessage> {
        let depth = allotments.map(|a| a.depth());
        self.extend(RectangleShape::<AllotmentRequest>::new(area,depth,patina,allotments)?);
        Ok(())
    }

    pub fn add_text(&mut self, position: HoleySpaceBase<f64>, pen: Pen, text: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>) -> Result<(),DataMessage> {
        let depth = allotments.map(|a| a.depth());
        self.extend(TextShape::<AllotmentRequest>::new(position,depth,pen,text,allotments)?);
        Ok(())
    }

    pub fn add_image(&mut self, position: HoleySpaceBase<f64>, images: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>) -> Result<(),DataMessage> {
        let depth = allotments.map(|a| a.depth());
        self.extend(ImageShape::<AllotmentRequest>::new(position,depth,images,allotments)?);
        Ok(())
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: AllotmentRequest) -> Result<(),DataMessage> {
        let depth = EachOrEvery::Every(allotment.depth());
        self.extend(WiggleShape::<AllotmentRequest>::new((min,max),values,depth,plotter,allotment.clone())?);
        Ok(())
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> ShapeListBuilder {
        let mut shapes = vec![];
        for shape in self.shapes.iter() {
            shapes.push(shape.filter_by_minmax(min_value,max_value));
        }
        ShapeListBuilder { shapes, allotments: self.allotments.clone(), universe: self.universe.clone(), assets: self.assets.clone() }
    }

    pub fn append(&mut self, more: &ShapeListBuilder) {
        self.shapes.extend(more.shapes.iter().cloned());
        self.allotments = self.allotments.union(&more.allotments).cloned().collect();
        self.universe.union(&more.universe);
    }
}

#[derive(Clone)]
pub struct CarriageShapeList3 {
    shapes: Arc<Vec<Shape<AllotmentRequest>>>,
    universe: Universe
}

impl CarriageShapeList3 {
    pub fn empty() -> CarriageShapeList3 {
        CarriageShapeList3 {
            shapes: Arc::new(vec![]),
            universe: Universe::new(&AllotmentMetadataStore::new())
        }
    }

    pub fn universe(&self) -> &Universe { &self.universe }
    pub fn len(&self) -> usize { self.shapes.len() }
    pub fn shapes(&self) -> Arc<Vec<Shape<AllotmentRequest>>> { self.shapes.clone() }

    pub fn new(builder: ShapeListBuilder, extent: Option<&CarriageExtent>) -> CarriageShapeList3 {
        builder.universe.allot(extent);
        CarriageShapeList3 {
            universe: builder.universe.clone(),
            shapes: Arc::new(builder.shapes)
        }
    }
}

#[derive(Clone)]
pub struct CarriageShapeList {
    shapes: Arc<Vec<Shape<Allotment>>>,
    universe: Universe
}

impl CarriageShapeList {
    pub fn empty() -> CarriageShapeList {
        CarriageShapeList {
            shapes: Arc::new(vec![]),
            universe: Universe::new(&AllotmentMetadataStore::new())
        }
    }

    pub fn universe(&self) -> &Universe { &self.universe }
    pub fn len(&self) -> usize { self.shapes.len() }
    pub fn shapes(&self) -> Arc<Vec<Shape<Allotment>>> { self.shapes.clone() }

    pub fn new(mut builder: ShapeListBuilder, extent: Option<&CarriageExtent>) -> Result<CarriageShapeList,DataMessage> {
        /* allotments are assigned here */
        builder.universe.allot(extent);
        /* shapes mapped to allotments here */
        let shapes = builder.shapes.drain(..).map(|s| s.allot(|r| r.allotment())).collect::<Result<Vec<_>,_>>()?;
        Ok(CarriageShapeList {
            universe: builder.universe.clone(),
            shapes: Arc::new(shapes)
        })
    }
}
