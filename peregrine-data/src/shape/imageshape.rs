use peregrine_toolkit::puzzle::PuzzleSolution;

use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Shape, ShapeDemerge, ShapeDetails, shape::shape::ShapeCommon, util::eachorevery::eoe_throw, SpaceBase, allotment::{transform_spacebase2, tree::allotmentbox::AllotmentBox, transformers::transformers::{Transformer, TransformerVariety}}};
use std::{hash::Hash, sync::Arc};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ImageShape<A> {
    position: SpaceBase<f64,A>,
    names: EachOrEvery<String>
}

impl<A> ImageShape<A> {
    pub fn map_new_allotment<F,B>(&self, cb: F) -> ImageShape<B> where F: Fn(&A) -> B {
        ImageShape {
            position: self.position.map_allotments(cb),
            names: self.names.clone()
        }
    }
}

impl<A> Clone for ImageShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { position: self.position.clone(), names: self.names.clone() }
    }
}

impl<A: Clone> ImageShape<A> {
    pub fn new_details(position: SpaceBase<f64,A>, names: EachOrEvery<String>) -> Option<ImageShape<A>> {
        if !names.compatible(position.len()) { return None; }
        Some(ImageShape {
            position, names
        })
    }

    pub fn new(position: SpaceBase<f64,AllotmentRequest>, depth: EachOrEvery<i8>, names: EachOrEvery<String>) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        let len = position.len();
        let mut out = vec![];
        let demerge = position.demerge_by_allotment(|x| { x.coord_system() });
        let details = eoe_throw("add_image",ImageShape::new_details(position,names))?;
        for (coord_system,mut filter) in demerge {
            filter.set_size(len);
            out.push(Shape::new(
                eoe_throw("add_image",ShapeCommon::new(coord_system,depth.clone()))?,
                ShapeDetails::Image(details.filter(&mut filter))
            ));
        }
        Ok(out)        
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn names(&self) -> &EachOrEvery<String> { &self.names }
    pub fn position(&self) -> &SpaceBase<f64,A> { &self.position }

    pub fn make_base_filter(&self, min: f64, max: f64) -> DataFilter {
        self.position.make_base_filter(min,max)
    }

    pub(super) fn filter(&self, filter: &DataFilter) -> ImageShape<A> {
        ImageShape {
            position: self.position.filter(filter),
            names: self.names.filter(&filter)
        }
    }

    pub fn iter_names(&self) -> impl Iterator<Item=&String> {
        self.names.iter(self.position.len()).unwrap()
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon, cat: &D) -> Vec<(T,ShapeCommon,ImageShape<A>)> where D: ShapeDemerge<X=T> {
        let demerge = self.position.allotments().demerge(|_| cat.categorise(common_in.coord_system()));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            let common = common_in.filter(&filter);
            filter.set_size(self.position.len());
            out.push((draw_group,common,self.filter(&filter)));
        }
        out
    }
}

impl ImageShape<Arc<dyn Transformer>> {
    fn demerge_by_variety(&self) -> Vec<(TransformerVariety,ImageShape<Arc<dyn Transformer>>)> {
        let demerge = self.position.allotments().demerge(|x| {
            x.choose_variety()
        });
        let mut out = vec![];
        for (variety,mut filter) in demerge {
            filter.set_size(self.position.len());
            out.push((variety,self.filter(&filter)));
        }
        out
    }
}

impl ImageShape<AllotmentRequest> {
    pub fn allot<F,E>(self, cb: F) -> Result<ImageShape<AllotmentBox>,E> where F: Fn(&AllotmentRequest) -> Result<AllotmentBox,E> {
        Ok(ImageShape {
            position: self.position.fullmap_allotments_results(cb)?,
            names: self.names.clone(),
        })
    }
}

impl ImageShape<AllotmentBox> {
    pub fn transform(&self, common: &ShapeCommon, solution: &PuzzleSolution) -> ImageShape<()> {
        ImageShape {
            position: transform_spacebase2(solution,&common.coord_system(),&self.position),
            names: self.names.clone()
        }
    }
}

impl ImageShape<Arc<dyn Transformer>> {
    pub fn make(&self, solution: &PuzzleSolution, common: &ShapeCommon) -> Vec<ImageShape<()>> {
        let mut out = vec![];
        for (variety,rectangles) in self.demerge_by_variety() {
            out.push(ImageShape {
                position: variety.spacebase_transform(&common.coord_system(),&self.position),
                names: self.names.clone()
            });
        }
        out
    }
}
