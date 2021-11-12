use std::hash::Hash;
use crate::{Colour, DataFilter, EachOrEvery, Flattenable, HoleySpaceBaseArea, ShapeCommon, ShapeDemerge, SpaceBaseArea};

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct LineShape {
    line: HoleySpaceBaseArea,
    colour: EachOrEvery<Colour>,
    width: u32
}

impl LineShape {
    pub fn len(&self) -> usize { self.line.len() }

    pub(super) fn filter(&self, filter: &DataFilter) -> LineShape {
        LineShape {
            line: self.line.filter(filter),
            colour: self.colour.filter(filter),
            width: self.width.clone()
        }
    }

    pub fn holey_line(&self) -> &HoleySpaceBaseArea { &self.line }
    pub fn line(&self) -> SpaceBaseArea<f64> { self.line.extract().0 }
    pub fn width(&self) -> u32 { self.width }
    pub fn colour(&self) -> &EachOrEvery<Colour> { &self.colour }

    pub fn make_base_filter(&self, min: f64, max: f64) -> DataFilter {
        self.line.make_base_filter(min,max)
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon, cat: &D) -> Vec<(T,ShapeCommon,LineShape)> where D: ShapeDemerge<X=T> {
        let allotments_and_colours = common_in.allotments().merge(&self.colour).unwrap();
        let demerge = allotments_and_colours.demerge(|(a,c)| 
            cat.categorise_colour(a,c)
        );
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            filter.set_size(self.line.len());
            let common = common_in.filter(&filter);
            out.push((draw_group,common,self.filter(&filter)));
        }
        out
    }
}
