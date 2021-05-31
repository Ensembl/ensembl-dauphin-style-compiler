use std::sync::Arc;
use crate::util::ringarray::{ UniformData, DataFilter };

fn cycle<T>(data: &[T], index: usize) -> &T {
    &data[index%data.len()]
}

pub struct SpaceBasePoint {
    base: f64,
    normal: f64,
    tangent: f64
}

#[derive(Debug)]
pub struct SpaceBasePointRef<'a> {
    pub base: &'a f64,
    pub normal: &'a f64,
    pub tangent: &'a f64
}

impl<'a> SpaceBasePointRef<'a> {
    fn make(&self) -> SpaceBasePoint {
        SpaceBasePoint {
            base: *self.base,
            normal: *self.normal,
            tangent: *self.tangent
        }
    }
}

/* If any are empty, all are empty */

#[derive(Debug)]
pub struct SpaceBase {
    base: Arc<Vec<f64>>,
    normal: Arc<Vec<f64>>,
    tangent: Arc<Vec<f64>>,

    max_len: usize
}

impl Clone for SpaceBase {
    fn clone(&self) -> Self {
        SpaceBase {
            base: self.base.clone(),
            normal: self.normal.clone(),
            tangent: self.tangent.clone(),
            max_len: self.max_len
        }
    }
}

impl SpaceBase {
    pub fn empty() -> SpaceBase {
        SpaceBase {
            base: Arc::new(vec![]),
            normal: Arc::new(vec![]),
            tangent: Arc::new(vec![]),
            max_len: 0
        }
    }

    pub fn len(&self) -> usize { self.max_len }

    pub fn new(base: Vec<f64>, normal: Vec<f64>, tangent: Vec<f64>) -> SpaceBase {
        let max_len = base.len().max(normal.len()).max(tangent.len());
        if base.len() == 0 || normal.len() == 0 || tangent.len() == 0 {
            SpaceBase::empty()
        } else {
            SpaceBase {
                base: Arc::new(base),
                normal: Arc::new(normal),
                tangent: Arc::new(tangent),
                max_len
            }
        }
    }

    pub fn iter_len<'a>(&'a self, length: usize) -> SpaceBaseIterator<'a> {
        SpaceBaseIterator {
            spacebase: self,
            index: 0,
            length
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> SpaceBase {
        SpaceBase {
            base: Arc::new(filter.filter(&self.base)),
            normal: Arc::new(filter.filter(&self.normal)),
            tangent: Arc::new(filter.filter(&self.tangent)),
            max_len: filter.count()
        }
    }

    pub fn make_base_filter(&self, min_value: f64, max_value: f64) -> DataFilter {
        let mut filter = DataFilter::new(&mut self.base.iter(),|base| {
            let exclude =  *base >= max_value || *base < min_value;
            !exclude
        });
        filter.set_size(self.max_len);
        filter
    }

    pub fn delta(&self, x_size: &[f64], y_size: &[f64]) -> SpaceBase {
        if x_size.len() == 0 || y_size.len() == 0 {
            return SpaceBase::empty()
        }
        let mut x_iter = x_size.iter().cycle();
        let mut y_iter = y_size.iter().cycle();
        SpaceBase {
            base: self.base.clone(),
            tangent: Arc::new(self.tangent.iter().map(|x| *x+x_iter.next().unwrap()).collect()),
            normal: Arc::new(self.normal.iter().map(|y| *y+y_iter.next().unwrap()).collect()),
            max_len: self.max_len
        }
    }
}

pub struct SpaceBaseIterator<'a> {
    spacebase: &'a SpaceBase,
    index: usize,
    length: usize
}

impl<'a> Iterator for SpaceBaseIterator<'a> {
    type Item = SpaceBasePointRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length { return None; }
        let out = SpaceBasePointRef {
            base: cycle(&self.spacebase.base,self.index),
            normal: cycle(&self.spacebase.normal,self.index),
            tangent: cycle(&self.spacebase.tangent,self.index),
        };
        self.index += 1;
        Some(out)
    }
}

pub struct SpaceBaseAreaIterator<'a> {
    a: SpaceBaseIterator<'a>,
    b: SpaceBaseIterator<'a>,
}

impl<'a> Iterator for SpaceBaseAreaIterator<'a> {
    type Item = (SpaceBasePointRef<'a>,SpaceBasePointRef<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (x,y) = (self.a.next(),self.b.next());
        if x.is_none() || y.is_none() { return None; }
        Some((x.unwrap(),y.unwrap()))
    }
}

#[derive(Debug)]
pub struct SpaceBaseArea(SpaceBase,SpaceBase);

impl SpaceBaseArea {
    pub fn new(top_left: SpaceBase, bottom_right: SpaceBase) -> SpaceBaseArea {
        SpaceBaseArea(top_left,bottom_right)
    }

    pub fn new_from_sizes(points: &SpaceBase, x_size: &[f64], y_size: &[f64]) -> SpaceBaseArea {
        SpaceBaseArea(points.clone(),points.delta(x_size,y_size))
    }

    pub fn len(&self) -> usize {  self.0.max_len.max(self.1.max_len) }

    pub fn iter(&self) -> SpaceBaseAreaIterator {
        let len = self.0.max_len.max(self.1.max_len);
        SpaceBaseAreaIterator {
            a: self.0.iter_len(len),
            b: self.1.iter_len(len),
        }
    }

    pub fn iter_other<'a,X>(&self, other: &'a [X]) -> impl Iterator<Item=&'a X> {
        let len = self.len();
        other.iter().cycle().take(len)
    }
}

impl SpaceBaseArea {
    pub fn make_base_filter(&self, min_value: f64, max_value: f64) -> DataFilter {
        let top_left = DataFilter::new(&mut self.0.base.iter(),|base| {
            *base <= max_value
        });
        let bottom_right = DataFilter::new(&mut self.1.base.iter(),|base| {
            *base >= min_value
        });
        top_left.and(&bottom_right)
    }

    pub fn filter(&self, filter: &DataFilter) -> SpaceBaseArea {
        SpaceBaseArea(self.0.filter(filter),self.1.filter(filter))
    }
}

impl Clone for SpaceBaseArea {
    fn clone(&self) -> Self {
        SpaceBaseArea(self.0.clone(),self.1.clone())
    }
}
