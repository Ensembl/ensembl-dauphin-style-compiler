use std::{ops::Range};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct HotspotPosition {
    pub top: f64,
    pub left: f64,
    pub right: f64,
    pub bottom: f64
}

pub trait HotspotStoreProfile<V> {
    type Context;
    type Area;

    fn diagonalise(&self, x: usize, y: usize) -> usize;
    fn get_zones(&self, context: &Self::Context, coords: &(f64,f64)) -> Vec<(usize,usize)>;
    fn bounds(&self, context: &Self::Context, value: &V) -> Option<HotspotPosition>;
    fn add_zones(&self, a: &Self::Area) -> Option<(Range<usize>,Range<usize>)>;
}

pub struct HotspotStore<A,X,V> {
    profile: Box<dyn HotspotStoreProfile<V,Context=X,Area=A>>,
    data: Vec<Option<Vec<usize>>>,
    values: Vec<V>
}

impl<A,X,V> HotspotStore<A,X,V> {
    pub fn new(profile: Box<dyn HotspotStoreProfile<V,Context=X,Area=A>>) -> HotspotStore<A,X,V> {
        HotspotStore {
            profile,
            data: vec![],
            values: vec![]
        }
    }

    pub fn add(&mut self, a: &A, value: V) {
        let index = self.values.len();
        self.values.push(value);
        if let Some((x_range,y_range)) = self.profile.add_zones(a) {
            let c_max = self.profile.diagonalise(x_range.end-1,y_range.end-1);
            if c_max >= self.data.len() {
                self.data.resize_with(c_max+1,Default::default);
            }
            for y in y_range {
                for x in x_range.clone() {
                    let c = self.profile.diagonalise(x,y);
                    self.data[c].get_or_insert(vec![]).push(index);
                }
            }    
        }
    }

    pub fn any(&self, context: &X, coord: &(f64,f64)) -> bool {
        self.get(context,coord).len() != 0
    }

    pub fn get<'b>(&'b self, context: &X, coord: &(f64,f64)) -> Vec<(&'b V,HotspotPosition)> {
        let mut out = vec![];
        for (x,y) in self.profile.get_zones(context,coord) {
            if let Some(indexes) = self.data.get(self.profile.diagonalise(x,y)).map(|x| x.as_ref()).flatten() {
                let more = indexes.iter()
                    .map(|v| &self.values[*v])
                    .filter_map(|v| {
                        if let Some(p) = self.profile.bounds(context,v) {
                            if coord.0 >= p.left && coord.0 < p.right && coord.1 >= p.top && coord.1 < p.bottom {
                                return Some((v,p));
                            }
                        }
                        None
                    });
                out.extend(more);
            }
        }
        out
    }
}
