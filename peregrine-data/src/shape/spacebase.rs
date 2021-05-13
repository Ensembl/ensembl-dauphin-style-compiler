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
    base: &'a f64,
    normal: &'a f64,
    tangent: &'a f64
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

#[derive(Debug)]
pub struct SpaceBaseBuilder {
    base: UniformData<f64>,
    normal: UniformData<f64>,
    tangent: UniformData<f64>,

    max_len: usize
}

impl SpaceBaseBuilder {
    pub fn empty() -> SpaceBaseBuilder {
        SpaceBaseBuilder {
            base: UniformData::None,
            normal: UniformData::None,
            tangent: UniformData::None,
            max_len: 0
        }
    }

    pub fn add(&mut self, point: SpaceBasePoint) {
        self.base.add(point.base);
        self.normal.add(point.normal);
        self.tangent.add(point.tangent);
        self.max_len += 1;
    }

    pub fn build(self) -> SpaceBase {
        SpaceBase::new(
            self.base.get_compact(),
            self.normal.get_compact(),
            self.tangent.get_compact())
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
            max_len: filter.len()
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

    pub fn iter(&self) -> SpaceBaseAreaIterator {
        let len = self.0.max_len.max(self.1.max_len);
        SpaceBaseAreaIterator {
            a: self.0.iter_len(len),
            b: self.1.iter_len(len),
        }
    }

    pub fn iter_other<'a,X>(&self, other: &'a [X]) -> impl Iterator<Item=&'a X> {
        let len = self.0.max_len.max(self.1.max_len);
        other.iter().cycle().take(len)
    }
}

impl SpaceBaseArea {
    pub fn make_base_filter(&self, min_value: f64, max_value: f64) -> DataFilter {
        let mut uniform = UniformData::None;
        for (top_left,bottom_right) in self.iter() {
            let exclude = *top_left.base >= max_value || *bottom_right.base < min_value;
            uniform.add(!exclude);
        }
        DataFilter::new(uniform)
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
