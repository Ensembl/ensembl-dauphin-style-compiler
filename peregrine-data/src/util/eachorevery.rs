use core::fmt;
use std::{hash::Hash, sync::Arc};
use crate::{DataFilter, DataMessage};

pub struct EachOrEveryIterator<'a,X> {
    obj: &'a EachOrEvery<X>,
    index: usize,
    len: usize
}

impl<'a,X> Iterator for EachOrEveryIterator<'a,X> {
    type Item = &'a X;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.len { return None; }
        let out = match self.obj {
            EachOrEvery::Each(v) => &v[self.index],
            EachOrEvery::Every(v) => v
        };
        self.index += 1;
        Some(out)
    }
}

pub enum EachOrEvery<X> {
    Each(Arc<Vec<X>>),
    Every(X)
}

impl<X> EachOrEvery<X> {
    pub fn each(data: Vec<X>) -> EachOrEvery<X> {
        EachOrEvery::Each(Arc::new(data))
    }

    pub fn every(data: X) -> EachOrEvery<X> {
        EachOrEvery::Every(data)
    }

    pub fn compatible(&self, len: usize) -> bool {
        match self {
            EachOrEvery::Each(v) => v.len() == len,
            EachOrEvery::Every(_) => true
        }
    }

    pub fn empty(&self) -> bool {
        match self {
            EachOrEvery::Each(x) => x.len() == 0,
            EachOrEvery::Every(_) => false
        }
    }

    pub fn get(&self, index: usize) -> Option<&X> {
        match self {
            EachOrEvery::Each(x) => x.get(index),
            EachOrEvery::Every(x) => Some(x)
        }
    }

    pub fn iter<'a>(&'a self, len: usize) -> Option<impl Iterator<Item=&'a X>> {
        match self {
            EachOrEvery::Each(v) => {
                if v.len() != len {
                    return None;
                }
            }
            _ => {}
        }
        Some(EachOrEveryIterator {
            obj: self,
            index: 0,
            len
        })
    }

    pub fn map<F,Y>(&self, mut f: F) -> EachOrEvery<Y> where F: FnMut(&X) -> Y {
        match self {
            EachOrEvery::Each(v) => EachOrEvery::each(v.iter().map(|x| f(x)).collect()),
            EachOrEvery::Every(v) => EachOrEvery::Every(f(&v))
        }
    }

    pub fn map_results<F,Y,E>(&self, mut f: F) -> Result<EachOrEvery<Y>,E> where F: FnMut(&X) -> Result<Y,E> {
        Ok(match self {
            EachOrEvery::Each(v) => EachOrEvery::each(v.iter().map(|x| f(x)).collect::<Result<_,_>>()?),
            EachOrEvery::Every(v) => EachOrEvery::Every(f(&v)?)
        })
    }
}

impl<X: Clone> Clone for EachOrEvery<X> {
    fn clone(&self) -> Self {
        match self {
            Self::Each(arg0) => Self::Each(arg0.clone()),
            Self::Every(arg0) => Self::Every(arg0.clone()),
        }
    }
}

impl<X> EachOrEvery<X> where X: Clone {
    pub fn xxx_to_vec(&self) -> Vec<X> {
        match self {
            EachOrEvery::Each(x) => x.iter().cloned().collect(),
            EachOrEvery::Every(x) => vec![x.clone()]
        }
    }

    pub fn merge<Y: Clone>(&self, other: &EachOrEvery<Y>) -> Option<EachOrEvery<(X,Y)>> {
        match (self,other) {
            (EachOrEvery::Each(x),EachOrEvery::Each(y)) => {
                if x.len() != y.len() { return None; }
                Some(EachOrEvery::each(x.iter().zip(y.iter()).map(|(x,y)| (x.clone(),y.clone())).collect()))
            },
            (EachOrEvery::Each(x),EachOrEvery::Every(y)) => {
                Some(EachOrEvery::each(x.iter().map(|x| (x.clone(),y.clone())).collect()))
            },
            (EachOrEvery::Every(x),EachOrEvery::Each(y)) => {
                Some(EachOrEvery::each(y.iter().map(|y| (x.clone(),y.clone())).collect()))
            },
            (EachOrEvery::Every(x),EachOrEvery::Every(y)) => {
                Some(EachOrEvery::Every((x.clone(),y.clone())))
            }
        }
    }

    pub fn filter(&self, data_filter: &DataFilter) -> EachOrEvery<X> {
        match self {
            EachOrEvery::Each(v) => { EachOrEvery::each(data_filter.filter(&v)) },
            EachOrEvery::Every(v) => { EachOrEvery::Every(v.clone()) }
        }
    }

    pub fn new_filter<F>(&self, count: usize, cb: F) -> DataFilter  where F: Fn(&X) -> bool {
        match self {
            EachOrEvery::Each(v) => DataFilter::new(&mut v.iter(),cb),
            EachOrEvery::Every(v) => {
                if cb(v) { DataFilter::all(count) } else { DataFilter::empty(count) }
            }
        }
    }

    pub fn demerge<F,K: Hash+PartialEq+Eq>(&self,cb: F) -> Vec<(K,DataFilter)> where F: Fn(&X) -> K {
        match self {
            EachOrEvery::Each(v) => {
                DataFilter::demerge(v,cb)
            },
            EachOrEvery::Every(v) => {
                DataFilter::demerge(&[v.clone()],cb)
            }
        }
    }
}

impl<X: fmt::Debug> fmt::Debug for EachOrEvery<X> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Each(arg0) => f.debug_tuple("Each").field(arg0).finish(),
            Self::Every(arg0) => f.debug_tuple("Every").field(arg0).finish(),
        }
    }
}

pub fn eoe_throw<X>(kind: &str,input: Option<X>) -> Result<X,DataMessage> {
    input.ok_or_else(|| DataMessage::LengthMismatch(kind.to_string()))
}
