use std::collections::HashMap;
use std::iter;
use std::hash::Hash;

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

    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item=&A> + 'a> {
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

pub struct DataFilter(UniformData<bool>);

impl DataFilter {
    pub fn new(uniform: UniformData<bool>) -> DataFilter {
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
}
