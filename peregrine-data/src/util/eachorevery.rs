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

pub enum EachOrEveryMut<'a,X> {
    Each(&'a mut Vec<X>),
    Every(&'a mut X)
}

impl<'a,X> EachOrEveryMut<'a,X> {
    pub fn map<F>(&mut self, mut cb: F) where F: FnMut(&mut X) {
        match self {
            EachOrEveryMut::Each(items) => {
                for item in items.iter_mut() {
                    cb(item);
                }
            },
            EachOrEveryMut::Every(x) => {
                cb(x);
            }
        }
    }
}

pub struct EachOrEveryBuilder<X>(EachOrEvery<X>);

impl<X: Clone> EachOrEveryBuilder<X> {
    pub fn as_mut<'a>(&'a mut self) -> EachOrEveryMut<'a,X> {
        match &mut self.0 {
            EachOrEvery::Each(x) => EachOrEveryMut::Each(Arc::make_mut(x)),
            EachOrEvery::Every(x) => EachOrEveryMut::Every(x)
        }
    }

    pub fn make(self) -> EachOrEvery<X> { self.0 }
}

#[derive(Clone)]
pub enum EachOrEveryGroupCompatible {
    Any,
    Require(usize),
    Invalid
}

impl EachOrEveryGroupCompatible {
    pub fn new(len: Option<usize>) -> EachOrEveryGroupCompatible {
        if let Some(len) = len { EachOrEveryGroupCompatible::Require(len) } else { EachOrEveryGroupCompatible::Any }
    }

    pub fn add<T: Clone>(&mut self, item: &EachOrEvery<T>) -> EachOrEveryGroupCompatible {
        *self = match (self.clone(),item.len()) {
            (EachOrEveryGroupCompatible::Any,Some(len)) => EachOrEveryGroupCompatible::Require(len),
            (EachOrEveryGroupCompatible::Require(len2),Some(len)) if len != len2 => {
                EachOrEveryGroupCompatible::Invalid
            },
            (x,_) => x.clone()
        };
        self.clone()
    }

    pub fn len(&self) -> Option<usize> {
        match self {
            EachOrEveryGroupCompatible::Require(x) => Some(*x),
            _ => None
        }
    }

    pub fn compatible(&self) -> bool {
        match self {
            EachOrEveryGroupCompatible::Invalid => false,
            _ => true
        }
    }

    pub fn complete(&self) -> bool {
        self.len().is_some()
    }
}

pub enum EachOrEvery<X> {
    Each(Arc<Vec<X>>),
    Every(X)
}

impl<X: Clone> EachOrEvery<X> {
    pub fn each(data: Vec<X>) -> EachOrEvery<X> {
        EachOrEvery::Each(Arc::new(data))
    }

    pub fn as_builder(&self) -> EachOrEveryBuilder<X> {
        EachOrEveryBuilder(self.clone())
    }

    pub fn every(data: X) -> EachOrEvery<X> {
        EachOrEvery::Every(data)
    }

    pub fn to_each(&self, len: usize) -> Option<EachOrEvery<X>> {
        Some(match self {
            EachOrEvery::Every(x) => EachOrEvery::Each(Arc::new(vec![x.clone();len])),
            EachOrEvery::Each(x) if x.len() == len => EachOrEvery::Each(x.clone()),
            _ => { return None; }
        })
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

    pub fn map<F,Y: Clone>(&self, mut f: F) -> EachOrEvery<Y> where F: FnMut(&X) -> Y {
        match self {
            EachOrEvery::Each(v) => EachOrEvery::each(v.iter().map(|x| f(x)).collect()),
            EachOrEvery::Every(v) => EachOrEvery::Every(f(&v))
        }
    }

    pub fn enumerated_map<F,Y: Clone>(&self, mut f: F) -> EachOrEvery<Y> where F: FnMut(usize,&X) -> Y {
        match self {
            EachOrEvery::Each(v) => EachOrEvery::each(v.iter().enumerate().map(|x| f(x.0,x.1)).collect()),
            EachOrEvery::Every(v) => EachOrEvery::Every(f(0,&v))
        }
    }

    pub fn map_results<F,Y: Clone,E>(&self, mut f: F) -> Result<EachOrEvery<Y>,E> where F: FnMut(&X) -> Result<Y,E> {
        Ok(match self {
            EachOrEvery::Each(v) => EachOrEvery::each(v.iter().map(|x| f(x)).collect::<Result<_,_>>()?),
            EachOrEvery::Every(v) => EachOrEvery::Every(f(&v)?)
        })
    }

    pub fn len(&self) -> Option<usize> {
        match  self {
            EachOrEvery::Each(x) => Some(x.len()),
            EachOrEvery::Every(_) => None
        }
    }
    
    pub fn zip<W,F>(&self, other: &EachOrEvery<X>, cb: F) -> Option<EachOrEvery<W>> where F: Fn(&X,&X) -> W {
        Some(match (self,other) {
            (EachOrEvery::Every(a),EachOrEvery::Every(b)) => EachOrEvery::Every(cb(a,b)),
            (EachOrEvery::Every(a),EachOrEvery::Each(b)) =>
                EachOrEvery::Each(Arc::new(b.iter().map(|b| cb(a,b)).collect::<Vec<W>>())),
            (EachOrEvery::Each(a),EachOrEvery::Every(b)) => 
                EachOrEvery::Each(Arc::new(a.iter().map(|a| cb(a,b)).collect::<Vec<W>>())),
            (EachOrEvery::Each(a),EachOrEvery::Each(b)) if a.len() == b.len() => 
                EachOrEvery::Each(Arc::new(
                    a.iter().zip(b.iter()).map(|(a,b)| cb(a,b)).collect::<Vec<W>>()
                )),
            _  => { return None; }
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
