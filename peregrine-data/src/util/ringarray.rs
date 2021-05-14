use std::collections::HashMap;
use std::iter;
use std::hash::Hash;

pub struct UniformDataIterator<'a,A: std::fmt::Debug> {
    uniform: &'a UniformData<A>,
    index: usize,
}

impl<'a,A: std::fmt::Debug> Iterator for UniformDataIterator<'a,A> {
    type Item = &'a A;

    fn next(&mut self) -> Option<&'a A> {
        self.index += 1;
        match self.uniform {
            UniformData::None => None,
            UniformData::Uniform(value,size) => {
                if self.index <= *size { Some(value) } else { None }
            }
            UniformData::Varied(values) => {
                if self.index <= values.len() { Some(&values[self.index-1]) } else { None }
            }
        }
    }
}

impl<'a,A: std::fmt::Debug> Clone for UniformDataIterator<'a,A> {
    fn clone(&self) -> UniformDataIterator<'a,A> {
        UniformDataIterator {
            uniform: self.uniform,
            index: 0
        }
    }
}

#[derive(Debug)]
pub enum UniformData<A: std::fmt::Debug> {
    None,
    Uniform(A,usize),
    Varied(Vec<A>)
}

impl<A: Clone+PartialEq+std::fmt::Debug> UniformData<A> {
    pub fn add(&mut self, more: A) {
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

    pub fn get_compact(self) -> Vec<A> {
        match self {
            UniformData::None => vec![],
            UniformData::Uniform(current,_) => vec![current],
            UniformData::Varied(values) => values
        }
    }

    pub fn iter<'a>(&'a self) -> UniformDataIterator<'a,A> {
        UniformDataIterator {
            uniform: &self,
            index: 0
        }
    }

    fn set_size(&mut self, len: usize) {
        match self {
            UniformData::None => {},
            UniformData::Uniform(_,size) => { *size = len },
            UniformData::Varied(values) => {
                if len > values.len() {
                    let orig = values.clone();
                    while len > values.len() {
                        values.append(&mut orig.clone());
                    }
                }
                if len < values.len() {
                    values.truncate(len);
                }
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            UniformData::None => 0,
            UniformData::Uniform(_,size) => *size,
            UniformData::Varied(values) => values.len()
        }
    }

    fn count<F>(&self,mut cb: F) -> usize where F: FnMut(&A) -> bool {
        match self {
            UniformData::None => 0,
            UniformData::Uniform(value,size) => if cb(value) { *size } else { 0 },
            UniformData::Varied(values) => {
                values.iter().map(|v| if cb(v) {1} else {0}).reduce(|a,b| a+b).unwrap_or(0)
            }
        }
    }
}

#[derive(Debug)]
pub struct DataFilter(UniformData<bool>);

impl DataFilter {
    pub fn new(uniform: UniformData<bool>) -> DataFilter {
        DataFilter(uniform)
    }

    pub fn set_size(&mut self, len: usize) {
        self.0.set_size(len)
    }

    pub fn new_filter<F,X>(data: &[X], cb: F) -> DataFilter where F: Fn(&X) -> bool {
        let mut uniform = UniformData::None;
        for value in data {
            uniform.add(cb(value));
        }
        DataFilter(uniform)
    }

    pub fn demerge<F,X,K: Hash+PartialEq+Eq>(data: &[X],cb: F) -> Vec<(K,DataFilter)> where F: Fn(&X) -> K {
        let mut position = HashMap::new();
        let mut filters = vec![];
        for value in data.iter() {
            let kind = cb(value);
            if position.get(&kind).is_none() {
                position.insert(kind,filters.len());
                let kind = cb(value);
                filters.push((kind,UniformData::None));
            }
        }
        for value in data.iter() {
            let kind = cb(value);
            let index = *position.get(&kind).unwrap();
            for (i,(_,filter)) in filters.iter_mut().enumerate() {
                filter.add(index==i);
            }
        }
        filters.drain(..).map(|(k,u)| (k,DataFilter(u))).collect::<Vec<(K,DataFilter)>>()
    }

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
        let values = other.iter().cycle();
        let mut out = vec![];
        for (pass,value) in self.0.iter().zip(values) {
           if *pass { out.push(value.clone()) };
        }
        out
    }

    pub fn len(&self) -> usize { self.0.len() }
    pub fn count(&self) -> usize { self.0.count(|f| *f) }
}
