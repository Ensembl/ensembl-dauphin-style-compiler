use std::{ops::Range};

pub trait HotspotStoreProfile<V> {
    type Context;
    type Coords;
    type Area;

    fn diagonalise(&self, x: usize, y: usize) -> usize;
    fn get_zones(&self, context: &Self::Context, coords: &Self::Coords) -> Vec<(usize,usize)>;
    fn intersects(&self, context: &Self::Context, coords: &Self::Coords, value: &V) -> bool;
    fn add_zones(&self, a: &Self::Area) -> Option<(Range<usize>,Range<usize>)>;
}

pub struct HotspotStore<C,A,X,V> {
    profile: Box<dyn HotspotStoreProfile<V,Context=X,Coords=C,Area=A>>,
    data: Vec<Option<Vec<usize>>>,
    values: Vec<V>
}

impl<C,A,X,V> HotspotStore<C,A,X,V> {
    pub fn new(profile: Box<dyn HotspotStoreProfile<V,Context=X,Coords=C,Area=A>>) -> HotspotStore<C,A,X,V> {
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

    pub fn any(&self, context: &X, coord: &C) -> bool {
        self.get(context,coord).len() != 0
    }

    pub fn get<'b>(&'b self, context: &X, coord: &C) -> Vec<&'b V> {
        let mut out = vec![];
        for (x,y) in self.profile.get_zones(context,coord) {
            if let Some(indexes) = self.data.get(self.profile.diagonalise(x,y)).map(|x| x.as_ref()).flatten() {
                let more = indexes.iter()
                    .map(|v| &self.values[*v])
                    .filter(|v| self.profile.intersects(context,coord,v));
                out.extend(more);
            }
        }
        out
    }
}
