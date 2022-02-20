use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Shape, ShapeDemerge, ShapeDetails, shape::shape::ShapeCommon, util::eachorevery::eoe_throw, Allotment, SpaceBase};
use std::hash::Hash;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ImageShape<A: Clone> {
    position: SpaceBase<f64,A>,
    names: EachOrEvery<String>
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
        let demerge = self.position.allotments().demerge(|a| cat.categorise(common_in.coord_system()));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            let common = common_in.filter(&filter);
            filter.set_size(self.position.len());
            out.push((draw_group,common,self.filter(&filter)));
        }
        out
    }
}

impl ImageShape<AllotmentRequest> {
    pub fn allot<F,E>(self, cb: F) -> Result<ImageShape<Allotment>,E> where F: Fn(&AllotmentRequest) -> Result<Allotment,E> {
        Ok(ImageShape {
            position: self.position.map_allotments_results(cb)?,
            names: self.names.clone(),
        })
    }
}