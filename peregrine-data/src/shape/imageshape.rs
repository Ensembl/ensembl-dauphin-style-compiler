use peregrine_toolkit::puzzle::PuzzleSolution;

use crate::{AllotmentRequest, DataMessage, ShapeDemerge, Shape, util::{eachorevery::EachOrEveryFilter}, SpaceBase, allotment::{transform_spacebase2, tree::allotmentbox::AllotmentBox, transformers::transformers::{Transformer, TransformerVariety}, style::{pendingleaf::PendingLeaf, style::LeafCommonStyle}}, EachOrEvery, CoordinateSystem};
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

    pub fn len(&self) -> usize { self.position.len() }
    pub fn position(&self) -> &SpaceBase<f64,A> { &self.position }

    pub fn iter_names(&self) -> impl Iterator<Item=&String> {
        self.names.iter(self.position.len()).unwrap()
    }

    pub fn new_details(position: SpaceBase<f64,A>, names: EachOrEvery<String>) -> Result<ImageShape<A>,DataMessage> {
        if !names.compatible(position.len()) { return Err(DataMessage::LengthMismatch(format!("image patina"))); }
        Ok(ImageShape {
            position, names
        })
    }

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> ImageShape<A> {
        ImageShape {
            position: self.position.filter(filter),
            names: self.names.filter(&filter)
        }
    }
}

impl<A> Clone for ImageShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { position: self.position.clone(), names: self.names.clone() }
    }
}

impl ImageShape<PendingLeaf> {
    pub fn new2(position: SpaceBase<f64,PendingLeaf>, names: EachOrEvery<String>) -> Result<Shape<PendingLeaf>,DataMessage> {
        let details = ImageShape::new_details(position,names.clone())?;
        Ok(Shape::Image(details))
    }
}

impl<A: Clone> ImageShape<A> {
    pub fn new(position: SpaceBase<f64,AllotmentRequest>, names: EachOrEvery<String>) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        let len = position.len();
        let mut out = vec![];
        let demerge = position.demerge_by_allotment(|x| { x.coord_system() });
        if let Ok(details) = ImageShape::new_details(position,names) {
            for (coord_system,mut filter) in demerge {
                out.push(Shape::Image(details.filter(&mut filter)));
            }    
        }
        Ok(out)
    }

    pub fn names(&self) -> &EachOrEvery<String> { &self.names }

    pub fn make_base_filter(&self, min: f64, max: f64) -> EachOrEveryFilter {
        self.position.make_base_filter(min,max)
    }
}

impl ImageShape<Arc<dyn Transformer>> {
    fn demerge_by_variety(&self) -> Vec<((TransformerVariety,CoordinateSystem),ImageShape<Arc<dyn Transformer>>)> {
        let demerge = self.position.allotments().demerge(self.position.len(),|x| {
            x.choose_variety()
        });
        let mut out = vec![];
        for (variety,mut filter) in demerge {
            out.push((variety,self.filter(&filter)));
        }
        out
    }
}

impl ImageShape<PendingLeaf> {
    pub fn base_filter(&self, min: f64, max: f64) -> ImageShape<PendingLeaf> {
        let non_tracking = self.position.allotments().make_filter(self.position.len(),|a| !a.leaf_style().coord_system.is_tracking());
        let filter = self.position.make_base_filter(min,max);
        self.filter(&filter.or(&non_tracking))
    }
}

impl ImageShape<LeafCommonStyle> {
    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,ImageShape<LeafCommonStyle>)> where D: ShapeDemerge<X=T> {
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

impl ImageShape<AllotmentRequest> {
    pub fn allot<F,E>(self, cb: F) -> Result<ImageShape<AllotmentBox>,E> where F: Fn(&AllotmentRequest) -> Result<AllotmentBox,E> {
        Ok(ImageShape {
            position: self.position.fullmap_allotments_results(cb)?,
            names: self.names.clone(),
        })
    }
}

/*
impl ImageShape<AllotmentBox> {
    pub fn transform(&self, common: &ShapeCommon, solution: &PuzzleSolution) -> ImageShape<()> {
        ImageShape {
            position: transform_spacebase2(solution,&common.coord_system(),&self.position),
            names: self.names.clone()
        }
    }
}
*/

impl ImageShape<Arc<dyn Transformer>> {
    pub fn make(&self, _solution: &PuzzleSolution) -> Vec<ImageShape<LeafCommonStyle>> {
        let mut out = vec![];
        for ((variety,coord_system),images) in self.demerge_by_variety() {
            out.push(ImageShape {
                position: variety.spacebase_transform(&coord_system,&self.position),
                names: images.names.clone()
            });
        }
        out
    }
}
