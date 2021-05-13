use std::sync::Arc;
use std::iter;

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
enum UniformData<A: std::fmt::Debug> {
    None,
    Uniform(A,usize),
    Varied(Vec<A>)
}

impl<A: Clone+PartialEq+std::fmt::Debug> UniformData<A> {
    fn add(&mut self, more: A) {
        match self {
            UniformData::None => { *self = UniformData::Uniform(more,1); },
            UniformData::Uniform(current,count) => {
                if *current == more {
                    *count += 1;
                } else {
                    let mut many = vec![current.clone();*count];
                    many.push(more);
                    *self = UniformData::Varied(many);
                }
            },
            UniformData::Varied(values) => { values.push(more); }
        }
    }

    fn get_compact(self) -> Vec<A> {
        match self {
            UniformData::None => vec![],
            UniformData::Uniform(current,_) => vec![current],
            UniformData::Varied(values) => values
        }
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item=&A> + 'a> {
        match self {
            UniformData::None => Box::new(iter::empty()),
            UniformData::Uniform(current,size) => Box::new(iter::repeat(current).take(*size)),
            UniformData::Varied(values) => Box::new(values.iter())
        }
    }

    fn len(&self) -> usize {
        match self {
            UniformData::None => 0,
            UniformData::Uniform(_,size) => *size,
            UniformData::Varied(values) => values.len()
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


pub struct DataFilter(UniformData<bool>);

impl DataFilter {
    pub fn filter<X: Clone>(&self, other: &[X]) -> Vec<X> {
        if other.len() == 0 { return vec![] }
        match &self.0 {
            UniformData::None => { return vec![]; },
            UniformData::Uniform(false,_) => { return vec![]; },
            UniformData::Uniform(true,_) => {
                if other.len() == 1 {
                    return vec![other[0].clone()];
                }
            },
            _ => {}
        }
        let mut values = other.iter().cycle();
        let mut out = vec![];
        for (pass,value) in self.0.iter().zip(values) {
           if *pass { out.push(value.clone()) };
        }
        out
    }

    pub fn len(&self) -> usize { self.0.len() }
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
}

impl SpaceBaseArea {
    pub fn make_base_filter(&self, min_value: f64, max_value: f64) -> DataFilter {
        let mut uniform = UniformData::None;
        for (top_left,bottom_right) in self.iter() {
            let exclude = *top_left.base >= max_value || *bottom_right.base < min_value;
            uniform.add(!exclude);
        }
        DataFilter(uniform)
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
