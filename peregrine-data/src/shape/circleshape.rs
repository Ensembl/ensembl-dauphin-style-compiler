use peregrine_toolkit::eachorevery::{EachOrEveryFilter, EachOrEvery};
use crate::{DataMessage, ShapeDemerge, Shape, SpaceBase, allotment::{style::{style::LeafStyle}, boxes::leaf::AnchoredLeaf, util::rangeused::RangeUsed}, LeafRequest, BackendNamespace, CoordinateSystem, Patina, reactive::Observable};
use std::{hash::Hash,};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct CircleShape<A> {
    position: SpaceBase<f64,A>,
    patina: Patina,
    radius: EachOrEvery<f64>,
    wobble: Option<SpaceBase<Observable<'static,f64>,()>>
}

impl<A> CircleShape<A> {
    pub(super) fn map_new_allotment<F,B>(&self, cb: F) -> CircleShape<B> where F: FnMut(&A) -> B {
        CircleShape {
            position: self.position.map_allotments(cb),
            patina: self.patina.clone(),
            radius: self.radius.clone(),
            wobble: self.wobble.clone()
        }
    }

    pub(super) fn len(&self) -> usize { self.position.len() }

    pub fn position(&self) -> &SpaceBase<f64,A> { &self.position }
    pub fn patina(&self) -> &Patina { &self.patina }
    pub fn radius(&self) -> &EachOrEvery<f64> { &self.radius }
    pub fn wobble(&self) -> &Option<SpaceBase<Observable<'static,f64>,()>> { &self.wobble }
    
    fn new_details(position: SpaceBase<f64,A>, radius: EachOrEvery<f64>, patina: Patina, wobble: Option<SpaceBase<Observable<'static,f64>,()>>) -> Result<CircleShape<A>,DataMessage> {
        if !patina.compatible(position.len()) { return Err(DataMessage::LengthMismatch(format!("image patina"))); }
        Ok(CircleShape {
            position, patina, radius, wobble
        })
    }

    fn filter(&self, filter: &EachOrEveryFilter) -> CircleShape<A> {
        CircleShape {
            position: self.position.filter(filter),
            radius: self.radius.filter(&filter),
            patina: self.patina.filter(&filter),
            wobble: self.wobble.as_ref().map(|w| w.filter(filter))
        }
    }
}

impl<A> Clone for CircleShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { position: self.position.clone(), patina: self.patina.clone(), radius: self.radius.clone(), wobble: self.wobble.clone() }
    }
}

impl CircleShape<LeafRequest> {
    pub fn new(position: SpaceBase<f64,LeafRequest>, radius: EachOrEvery<f64>, patina: Patina, wobble: Option<SpaceBase<Observable<'static,f64>,()>>) -> Result<Shape<LeafRequest>,DataMessage> {
        let details = CircleShape::new_details(position,radius,patina,wobble)?;
        Ok(Shape::Circle(details))
    }

    pub(super) fn base_filter(&self, min: f64, max: f64) -> CircleShape<LeafRequest> {
        let non_tracking = self.position.allotments().make_filter(self.position.len(),|a| !a.leaf_style().coord_system.is_tracking());
        let filter = self.position.make_base_filter(min,max);
        self.filter(&filter.or(&non_tracking))
    }

    pub(super) fn register_space(&self) {
        let position = self.position().iter();
        let radius = self.radius().iter(self.position.len()).unwrap();     
        for (position,radius) in position.zip(radius) {
            position.allotment.drawing_info(|allotment| {
                allotment.merge_base_range(&RangeUsed::Part(*position.base,*position.base));
                allotment.merge_pixel_range(&RangeUsed::Part(*position.tangent-*radius,*position.tangent+*radius));
                allotment.merge_max_y((position.normal+*radius).ceil());
            });
        }    
    }
}

impl CircleShape<AnchoredLeaf> {
    fn demerge_by_variety(&self) -> Vec<(CoordinateSystem,CircleShape<AnchoredLeaf>)> {
        let demerge = self.position.allotments().demerge(self.position.len(),|x| {
            x.coordinate_system().clone()
        });
        let mut out = vec![];
        for (coord,filter) in demerge {
            out.push((coord,self.filter(&filter)));
        }
        out
    }

    pub fn make(&self) -> Vec<CircleShape<LeafStyle>> {
        let mut out = vec![];
        for (coord_system,circles) in self.demerge_by_variety() {
            out.push(CircleShape {
                position: self.position.spacebase_transform(&coord_system),
                patina: self.patina.clone(),
                radius: self.radius.clone(),
                wobble: self.wobble.clone()
            });
        }
        out
    }
}

impl CircleShape<LeafStyle> {
    pub(crate) fn demerge<T: Hash + Clone + Eq,D>(self, cat: &D) -> Vec<(T,CircleShape<LeafStyle>)> where D: ShapeDemerge<X=T> {
        let demerge = self.position.allotments().demerge(self.position.len(),|x| {
            cat.categorise(&x.coord_system)
        });
        let mut out = vec![];
        for (draw_group,filter) in demerge {
            out.push((draw_group,self.filter(&filter)));
        }
        out
    }
}
