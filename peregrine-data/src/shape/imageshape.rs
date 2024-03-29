use eachorevery::{EachOrEvery, EachOrEveryFilter};

use crate::{DataMessage, ShapeDemerge, Shape, SpaceBase, allotment::{leafs::anchored::AnchoredLeaf}, LeafRequest, BackendNamespace, CoordinateSystem, AuxLeaf};
use std::{hash::Hash,};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ImageShape<A> {
    position: SpaceBase<f64,A>,
    channel: BackendNamespace,
    names: EachOrEvery<String>
}

impl<A> ImageShape<A> {
    pub(super) fn map_new_allotment<F,B>(&self, cb: F) -> ImageShape<B> where F: FnMut(&A) -> B {
        ImageShape {
            position: self.position.map_allotments(cb),
            channel: self.channel.clone(),
            names: self.names.clone()
        }
    }

    pub(super) fn len(&self) -> usize { self.position.len() }

    pub fn position(&self) -> &SpaceBase<f64,A> { &self.position }
    pub fn channel(&self) -> &BackendNamespace { &self.channel }
    
    pub fn iter_names(&self) -> impl Iterator<Item=&String> {
        self.names.iter(self.position.len()).unwrap()
    }

    fn new_details(position: SpaceBase<f64,A>, channel: &BackendNamespace, names: EachOrEvery<String>) -> Result<ImageShape<A>,DataMessage> {
        if !names.compatible(position.len()) { return Err(DataMessage::LengthMismatch(format!("image patina"))); }
        Ok(ImageShape {
            position, names, channel: channel.clone()
        })
    }

    fn filter(&self, filter: &EachOrEveryFilter) -> ImageShape<A> {
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

    pub(super) fn base_filter(&self, min: f64, max: f64) -> ImageShape<LeafRequest> {
        let non_tracking = self.position.allotments().make_filter(self.position.len(),|a| !a.leaf_style().aux.coord_system.is_tracking());
        let filter = self.position.make_base_filter(min,max);
        self.filter(&filter.or(&non_tracking))
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

    pub fn make(&self) -> Vec<ImageShape<AuxLeaf>> {
        let mut out = vec![];
        for (coord_system,images) in self.demerge_by_variety() {
            out.push(ImageShape {
                position: images.position.spacebase_transform(&coord_system),
                channel: self.channel.clone(),
                names: images.names.clone()
            });
        }
        out
    }
}

impl ImageShape<AuxLeaf> {
    pub(crate) fn demerge<T: Hash + Clone + Eq,D>(self, cat: &D) -> Vec<(T,ImageShape<AuxLeaf>)> where D: ShapeDemerge<X=T> {
        let demerge = self.position.allotments().demerge(self.position.len(),|x| {
            cat.categorise(&x.coord_system,x.depth)
        });
        let mut out = vec![];
        for (draw_group,filter) in demerge {
            out.push((draw_group,self.filter(&filter)));
        }
        out
    }
}
