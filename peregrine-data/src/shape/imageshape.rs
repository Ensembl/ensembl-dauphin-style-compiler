use peregrine_toolkit::eachorevery::{EachOrEveryFilter, EachOrEvery};
use crate::{DataMessage, ShapeDemerge, Shape, SpaceBase, allotment::{style::{style::LeafStyle}, boxes::leaf::AnchoredLeaf}, LeafRequest, BackendNamespace, CoordinateSystem};
use std::{hash::Hash,};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ImageShape<A> {
    position: SpaceBase<f64,A>,
    channel: BackendNamespace,
    names: EachOrEvery<String>
}

impl<A> ImageShape<A> {
    pub fn map_new_allotment<F,B>(&self, cb: F) -> ImageShape<B> where F: FnMut(&A) -> B {
        ImageShape {
            position: self.position.map_allotments(cb),
            channel: self.channel.clone(),
            names: self.names.clone()
        }
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn position(&self) -> &SpaceBase<f64,A> { &self.position }
    pub fn channel(&self) -> &BackendNamespace { &self.channel }

    pub fn iter_names(&self) -> impl Iterator<Item=&String> {
        self.names.iter(self.position.len()).unwrap()
    }

    pub fn new_details(position: SpaceBase<f64,A>, channel: &BackendNamespace, names: EachOrEvery<String>) -> Result<ImageShape<A>,DataMessage> {
        if !names.compatible(position.len()) { return Err(DataMessage::LengthMismatch(format!("image patina"))); }
        Ok(ImageShape {
            position, names, channel: channel.clone()
        })
    }

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> ImageShape<A> {
        ImageShape {
            position: self.position.filter(filter),
            names: self.names.filter(&filter),
            channel: self.channel.clone()
        }
    }
}

impl<A> Clone for ImageShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { position: self.position.clone(), names: self.names.clone(), channel: self.channel.clone() }
    }
}

impl ImageShape<LeafRequest> {
    pub fn new(position: SpaceBase<f64,LeafRequest>, channel: &BackendNamespace, names: EachOrEvery<String>) -> Result<Shape<LeafRequest>,DataMessage> {
        let details = ImageShape::new_details(position,channel,names.clone())?;
        Ok(Shape::Image(details))
    }

    pub fn base_filter(&self, min: f64, max: f64) -> ImageShape<LeafRequest> {
        let non_tracking = self.position.allotments().make_filter(self.position.len(),|a| !a.leaf_style().coord_system.is_tracking());
        let filter = self.position.make_base_filter(min,max);
        self.filter(&filter.or(&non_tracking))
    }
}

impl<A: Clone> ImageShape<A> {
    pub fn names(&self) -> &EachOrEvery<String> { &self.names }

    pub fn make_base_filter(&self, min: f64, max: f64) -> EachOrEveryFilter {
        self.position.make_base_filter(min,max)
    }
}

impl ImageShape<AnchoredLeaf> {
    fn demerge_by_variety(&self) -> Vec<(CoordinateSystem,ImageShape<AnchoredLeaf>)> {
        let demerge = self.position.allotments().demerge(self.position.len(),|x| {
            x.coordinate_system().clone()
        });
        let mut out = vec![];
        for (coord,filter) in demerge {
            out.push((coord,self.filter(&filter)));
        }
        out
    }
}

impl ImageShape<LeafStyle> {
    pub fn demerge<T: Hash + Clone + Eq,D>(self, cat: &D) -> Vec<(T,ImageShape<LeafStyle>)> where D: ShapeDemerge<X=T> {
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

impl ImageShape<AnchoredLeaf> {
    pub fn make(&self) -> Vec<ImageShape<LeafStyle>> {
        let mut out = vec![];
        for (coord_system,images) in self.demerge_by_variety() {
            out.push(ImageShape {
                position: self.position.spacebase_transform(&coord_system),
                channel: self.channel.clone(),
                names: images.names.clone()
            });
        }
        out
    }
}
