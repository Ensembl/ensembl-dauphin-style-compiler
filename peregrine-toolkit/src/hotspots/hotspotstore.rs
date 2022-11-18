/* We don't want to constrain the user in zmenu size, but also want to keep small values compact
 * and fast (ie vec not map), so we do it diagonally, like you do with enumerating the rationals:
 * 
 * 0136
 * 247
 * 58
 * 9
 * etc.
 * 
 * (x+y)(x+y+1)/2 + y as you are in diagonal number x+y and these start with nth triangle number
 * and increase by one with each y-coord
 */

use std::{ops::Range};

pub trait HotspotStoreProfile<V> {
    type Context;
    type Coords;
    type Area;

    fn get_zone(&self, context: &Self::Context, coords: &Self::Coords) -> Option<(usize,usize)>;
    fn intersects(&self, context: &Self::Context, coords: &Self::Coords, value: &V) -> bool;
    fn add_zones(&self, a: &Self::Area) -> (Range<usize>,Range<usize>);
}

pub struct HotSpotStore<C,A,X,V> {
    getter: Box<dyn HotspotStoreProfile<V,Context=X,Coords=C,Area=A>>,
    data: Vec<Option<Vec<usize>>>,
    values: Vec<V>
}

fn diagonal(x: usize, y: usize) -> usize { (x+y)*(x+y+1)/2+y }

impl<C,A,X,V> HotSpotStore<C,A,X,V> {
    pub fn new(getter: Box<dyn HotspotStoreProfile<V,Context=X,Coords=C,Area=A>>) -> HotSpotStore<C,A,X,V> {
        HotSpotStore {
            getter,
            data: vec![],
            values: vec![]
        }
    }

    pub fn add(&mut self, a: &A, value: V) {
        let index = self.values.len();
        self.values.push(value);
        let (x_range,y_range) = self.getter.add_zones(a);
        let c_max = diagonal(x_range.end-1,y_range.end-1);
        if c_max >= self.data.len() {
            self.data.resize_with(c_max+1,Default::default);
        }
        for y in y_range {
            for x in x_range.clone() {
                let c = diagonal(x,y);
                self.data[c].get_or_insert(vec![]).push(index);
            }
        }
    }

    pub fn any(&self, context: &X, coord: &C) -> bool {
        self.get(context,coord).len() != 0
    }

    pub fn get<'b>(&'b self, context: &X, coord: &C) -> Vec<&'b V> {
        if let Some((x,y)) = self.getter.get_zone(context,coord) {
            let indexes = self.data.get(diagonal(x,y)).map(|x| x.as_ref()).flatten();
            indexes.map(|vv| {
                vv.iter()
                    .map(|v| &self.values[*v])
                    .filter(|v| self.getter.intersects(context,coord,v))
                    .collect::<Vec<_>>()
            }).unwrap_or(vec![])
        } else {
            vec![]
        }
    }
}
