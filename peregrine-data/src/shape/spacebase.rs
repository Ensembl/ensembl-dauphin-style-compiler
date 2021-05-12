use std::sync::Arc;

fn cycle<T>(data: &[T], index: usize) -> &T {
    &data[index%data.len()]
}

pub struct SpaceBasePoint<A> {
    base: f64,
    space: A,
    normal: f64,
    tangent: f64
}

#[derive(Debug)]
pub struct SpaceBasePointRef<'a,A> {
    base: &'a f64,
    space: &'a A,
    normal: &'a f64,
    tangent: &'a f64
}

impl<'a,A: Clone> SpaceBasePointRef<'a,A> {
    fn make(&self) -> SpaceBasePoint<A> {
        SpaceBasePoint {
            base: *self.base,
            space: self.space.clone(),
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

    fn get(self) -> Vec<A> {
        match self {
            UniformData::None => vec![],
            UniformData::Uniform(current,_) => vec![current],
            UniformData::Varied(values) => values
        }
    }
}

#[derive(Debug)]
pub struct SpaceBaseBuilder<A: std::fmt::Debug> {
    base: UniformData<f64>,
    space: UniformData<A>,
    normal: UniformData<f64>,
    tangent: UniformData<f64>,

    max_len: usize
}

impl<A: PartialEq+Clone+std::fmt::Debug> SpaceBaseBuilder<A> {
    pub fn empty() -> SpaceBaseBuilder<A> {
        SpaceBaseBuilder {
            base: UniformData::None,
            space: UniformData::None,
            normal: UniformData::None,
            tangent: UniformData::None,
            max_len: 0
        }
    }

    pub fn add(&mut self, point: SpaceBasePoint<A>) {
        self.base.add(point.base);
        self.space.add(point.space);
        self.normal.add(point.normal);
        self.tangent.add(point.tangent);
        self.max_len += 1;
    }

    pub fn build(self) -> SpaceBase<A> {
        SpaceBase::new(
            self.base.get(),
            self.space.get(),
            self.normal.get(),
            self.tangent.get())
    }
}

/* If any are empty, all are empty */

#[derive(Debug)]
pub struct SpaceBase<A> {
    base: Arc<Vec<f64>>,
    space: Arc<Vec<A>>,
    normal: Arc<Vec<f64>>,
    tangent: Arc<Vec<f64>>,

    max_len: usize
}

impl<A: 'static> Clone for SpaceBase<A> {
    fn clone(&self) -> Self {
        SpaceBase {
            base: self.base.clone(),
            space: self.space.clone(),
            normal: self.normal.clone(),
            tangent: self.tangent.clone(),
            max_len: self.max_len
        }
    }
}

impl<A> SpaceBase<A> {
    pub fn empty() -> SpaceBase<A> {
        SpaceBase {
            base: Arc::new(vec![]),
            space: Arc::new(vec![]),
            normal: Arc::new(vec![]),
            tangent: Arc::new(vec![]),
            max_len: 0
        }
    }

    pub fn new(base: Vec<f64>, space: Vec<A>, normal: Vec<f64>, tangent: Vec<f64>) -> SpaceBase<A> {
        let max_len = base.len().max(space.len()).max(normal.len()).max(tangent.len());
        if base.len() == 0 || space.len() == 0 || normal.len() == 0 || tangent.len() == 0 {
            SpaceBase::empty()
        } else {
            SpaceBase {
                base: Arc::new(base),
                space: Arc::new(space),
                normal: Arc::new(normal),
                tangent: Arc::new(tangent),
                max_len
            }
        }
    }

    pub fn iter_len<'a>(&'a self, length: usize) -> SpaceBaseIterator<'a,A> {
        SpaceBaseIterator {
            spacebase: self,
            index: 0,
            length
        }
    }

    pub fn map_space<F,B>(&self, cb: &mut F) -> SpaceBase<B> where F: FnMut(&A) -> B {
        SpaceBase {
            base: self.base.clone(),
            space: Arc::new(self.space.iter().map(move |a| cb(a)).collect()),
            normal: self.normal.clone(),
            tangent: self.tangent.clone(),
            max_len: self.max_len
        }
    }

    pub fn try_map_space<F,B,E>(&self, cb: &mut F) -> Result<SpaceBase<B>,E> where F: FnMut(&A) -> Result<B,E> {
        Ok(SpaceBase {
            base: self.base.clone(),
            space: Arc::new(self.space.iter().map(move |a| cb(a)).collect::<Result<Vec<_>,_>>()?),
            normal: self.normal.clone(),
            tangent: self.tangent.clone(),
            max_len: self.max_len
        })
    }
}

pub struct SpaceBaseIterator<'a,A> {
    spacebase: &'a SpaceBase<A>,
    index: usize,
    length: usize
}

impl<'a,A> Iterator for SpaceBaseIterator<'a,A> {
    type Item = SpaceBasePointRef<'a,A>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length { return None; }
        let out = SpaceBasePointRef {
            base: cycle(&self.spacebase.base,self.index),
            space: cycle(&self.spacebase.space,self.index),
            normal: cycle(&self.spacebase.normal,self.index),
            tangent: cycle(&self.spacebase.tangent,self.index),
        };
        self.index += 1;
        Some(out)
    }
}

pub struct SpaceBaseAreaIterator<'a,A> {
    a: SpaceBaseIterator<'a,A>,
    b: SpaceBaseIterator<'a,A>
}

impl<'a,A> Iterator for SpaceBaseAreaIterator<'a,A> {
    type Item = (SpaceBasePointRef<'a,A>,SpaceBasePointRef<'a,A>);

    fn next(&mut self) -> Option<Self::Item> {
        let (x,y) = (self.a.next(),self.b.next());
        if x.is_none() || y.is_none() { return None; }
        Some((x.unwrap(),y.unwrap()))
    }
}

#[derive(Debug)]
pub struct SpaceBaseArea<A>(SpaceBase<A>,SpaceBase<A>);

impl<A> SpaceBaseArea<A> {
    pub fn new(top_left: SpaceBase<A>, bottom_right: SpaceBase<A>) -> SpaceBaseArea<A> {
        SpaceBaseArea(top_left,bottom_right)
    }

    pub fn iter(&self) -> SpaceBaseAreaIterator<A> {
        let len = self.0.max_len.max(self.1.max_len);
        SpaceBaseAreaIterator {
            a: self.0.iter_len(len),
            b: self.1.iter_len(len),
        }
    }

    pub fn map_space<F,B>(&self, mut cb: F) -> SpaceBaseArea<B> where F: FnMut(&A) -> B {
        let cb = &mut cb;
        SpaceBaseArea(self.0.map_space(cb),self.1.map_space(cb))
    }

    pub fn try_map_space<F,B,E>(&self, mut cb: &mut F) -> Result<SpaceBaseArea<B>,E> where F: FnMut(&A) -> Result<B,E> {
        let cb = &mut cb;
        Ok(SpaceBaseArea(self.0.try_map_space(cb)?,self.1.try_map_space(cb)?))
    }
}

impl<A: Clone+PartialEq+std::fmt::Debug> SpaceBaseArea<A> {
    pub fn filter_base(&self, min_value: f64, max_value: f64) -> SpaceBaseArea<A> {
        let mut top_left_builder = SpaceBaseBuilder::empty();
        let mut bottom_right_builder = SpaceBaseBuilder::empty();
        for (top_left,bottom_right) in self.iter() {
            if *top_left.base >= max_value || *bottom_right.base < min_value { continue; }
            top_left_builder.add(top_left.make());
            bottom_right_builder.add(bottom_right.make());
        }
        SpaceBaseArea(top_left_builder.build(),bottom_right_builder.build())
    }
}

impl<A: 'static> Clone for SpaceBaseArea<A> {
    fn clone(&self) -> Self {
        SpaceBaseArea(self.0.clone(),self.1.clone())
    }
}
