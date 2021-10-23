use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Flattenable, HoleySpaceBase, Shape, ShapeDemerge, ShapeDetails, SpaceBase, shape::shape::ShapeCommon, util::eachorevery::eoe_throw};
use std::hash::Hash;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ImageShape {
    position: HoleySpaceBase,
    names: EachOrEvery<String>
}

impl ImageShape {
    pub fn new_details(position: HoleySpaceBase, names: EachOrEvery<String>) -> Option<ImageShape> {
        if !names.compatible(position.len()) { return None; }
        Some(ImageShape {
            position, names
        })
    }

    pub fn new(position: HoleySpaceBase, names: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>) -> Result<Vec<Shape>,DataMessage> {
        let len = position.len();
        let mut out = vec![];
        let details = eoe_throw("add_image",ImageShape::new_details(position,names))?;
        for (coord_system,mut filter) in allotments.demerge(|x| { x.coord_system() }) {
            filter.set_size(len);
            out.push(Shape::new(
                eoe_throw("add_image",ShapeCommon::new(filter.count(),coord_system,allotments.filter(&filter)))?,
                ShapeDetails::Image(details.filter(&mut filter))
            ));
        }
        Ok(out)        
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn names(&self) -> &EachOrEvery<String> { &self.names }
    pub fn holey_position(&self) -> &HoleySpaceBase { &self.position }
    pub fn position(&self) -> SpaceBase<f64> { self.position.extract().0 }

    fn filter(&self, filter: &DataFilter) -> ImageShape {
        ImageShape {
            position: self.position.filter(filter),
            names: self.names.filter(&filter)
        }
    }

    pub fn iter_names(&self) -> impl Iterator<Item=&String> {
        self.names.iter(self.position.len()).unwrap()
    }

    pub fn filter_by_minmax(&self, common: &mut ShapeCommon, min: f64, max: f64) -> ImageShape {
        let mut filter = self.position.make_base_filter(min,max);
        filter.set_size(self.position.len());
        *common = common.filter(&filter);
        self.filter(&filter)
    }

    pub fn filter_by_allotment<F>(&self, common: &mut ShapeCommon, cb: F)  -> ImageShape where F: Fn(&AllotmentRequest) -> bool {
        let mut filter = common.allotments().new_filter(self.position.len(),cb);
        filter.set_size(self.position.len());
        *common = common.filter(&filter);
        self.filter(&filter)
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon, cat: &D) -> Vec<(T,ShapeCommon,ImageShape)> where D: ShapeDemerge<X=T> {
        let demerge = common_in.allotments().demerge(|a| cat.categorise(a));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            let common = common_in.filter(&filter);
            filter.set_size(self.position.len());
            out.push((draw_group,common,self.filter(&filter)));
        }
        out
    }
}
