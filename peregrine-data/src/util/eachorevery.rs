use core::fmt;
use std::{hash::Hash};
use crate::DataFilter;

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
    Each(Vec<X>),
    Every(X)
}

impl<X> EachOrEvery<X> {
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
            EachOrEvery::Each(v) => EachOrEvery::Each(v.iter().map(|x| f(x)).collect()),
            EachOrEvery::Every(v) => EachOrEvery::Every(f(&v))
        }
    }

    pub fn map_into<F,Y>(self, mut f: F) -> EachOrEvery<Y> where F: FnMut(X) -> Y {
        match self {
            EachOrEvery::Each(mut v) => EachOrEvery::Each(v.drain(..).map(|x| f(x)).collect()),
            EachOrEvery::Every(v) => EachOrEvery::Every(f(v))
        }
    }

    pub fn map_results<F,Y,E>(&self, mut f: F) -> Result<EachOrEvery<Y>,E> where F: FnMut(&X) -> Result<Y,E> {
        Ok(match self {
            EachOrEvery::Each(v) => EachOrEvery::Each(v.iter().map(|x| f(x)).collect::<Result<_,_>>()?),
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
    pub fn filter(&self, data_filter: &DataFilter) -> EachOrEvery<X> {
        match self {
            EachOrEvery::Each(v) => { EachOrEvery::Each(data_filter.filter(&v)) },
            EachOrEvery::Every(v) => { EachOrEvery::Every(v.clone()) }
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
