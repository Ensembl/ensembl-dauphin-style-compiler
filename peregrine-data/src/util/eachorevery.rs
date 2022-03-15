use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

fn un_rle<F>(input: &[(usize,usize)], cb: F) -> Arc<Vec<usize>> where F: Fn(usize) -> usize {
    let mut out = vec![];
    for (start,len) in input {
        for i in *start..(*start+*len) {
            out.push(cb(i));
        }
    }
    Arc::new(out)
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
enum EachOrEveryIndex {
    Unindexed,
    Indexed(Arc<Vec<usize>>),
    Every
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct EachOrEvery<X> {
    index: EachOrEveryIndex,
    data: Arc<Vec<X>>
}

impl<X> Clone for EachOrEvery<X> {
    fn clone(&self) -> Self {
        Self { index: self.index.clone(), data: self.data.clone() }
    }
}

impl<X> EachOrEvery<X> {
    pub fn each(data: Vec<X>) -> EachOrEvery<X> {
        EachOrEvery {
            index: EachOrEveryIndex::Unindexed,
            data: Arc::new(data)
        }
    }

    pub fn every(data: X) -> EachOrEvery<X> {
        EachOrEvery {
            index: EachOrEveryIndex::Every,
            data: Arc::new(vec![data])
        }
    }

    pub fn len(&self) -> Option<usize> {
        match &self.index {
            EachOrEveryIndex::Unindexed => Some(self.data.len()),
            EachOrEveryIndex::Indexed(index) => Some(index.len()),
            EachOrEveryIndex::Every => None
        }
    }

    pub fn get(&self, pos: usize) -> Option<&X> {
        match &self.index {
            EachOrEveryIndex::Unindexed => self.data.get(pos),
            EachOrEveryIndex::Indexed(index) => self.data.get(index[pos]),
            EachOrEveryIndex::Every => self.data.get(0)
        }
    }

    pub fn demerge<F,K: Hash+PartialEq+Eq>(&self, len: usize, cb: F) -> Vec<(K,EachOrEveryFilter)> where F: Fn(&X) -> K {
        match &self.index {
            EachOrEveryIndex::Unindexed => {
                let mut out = HashMap::new();
                for (i,value) in self.data.iter().enumerate() {
                    out.entry(cb(value)).or_insert_with(|| EachOrEveryFilterBuilder::new()).set(i);
                }
                out.drain().map(|(key,filter)| (key,filter.make(len))).collect::<Vec<_>>()
            },
            EachOrEveryIndex::Indexed(index) => {
                let mut out = HashMap::new();
                for (i,value) in index.iter().enumerate() {
                    out.entry(cb(&self.data[*value])).or_insert_with(|| EachOrEveryFilterBuilder::new()).set(i);
                }
                out.drain().map(|(key,filter)| (key,filter.make(len))).collect::<Vec<_>>()
            },
            EachOrEveryIndex::Every => vec![(cb(&self.data[0]),EachOrEveryFilter::all(len))]
        }
    }

    pub fn map<F,Y>(&self, f: F) -> EachOrEvery<Y> where F: FnMut(&X) -> Y {
        EachOrEvery {
            index: self.index.clone(),
            data: Arc::new(self.data.iter().map(f).collect())
        }
    }
    
    pub fn map_mut<F>(&mut self, f: F) where F: Fn(&X) -> X {
        self.data = Arc::new(self.data.iter().map(f).collect::<Vec<_>>());
    }

    pub fn fold_mut<F,Z>(&mut self, data: &[Z], f: F) where F: Fn(&X,&Z) -> X {
        self.data = Arc::new(self.data.iter().zip(data.iter()).map(|(x,z)| f(x,z)).collect::<Vec<_>>());
    }

    pub fn map_results<F,Y,E>(&self, f: F) -> Result<EachOrEvery<Y>,E> where F: FnMut(&X) -> Result<Y,E> {
        let data = self.data.iter().map(f).collect::<Result<_,_>>()?;
        Ok(EachOrEvery {
            index: self.index.clone(),
            data: Arc::new(data)
        })
    }

    pub fn fullmap_results<F,Y,E>(&self, mut f: F) -> Result<EachOrEvery<Y>,E> where F: FnMut(&X) -> Result<Y,E> {
        Ok(if let Some(len) = self.len() {
            let out = self.iter(len).unwrap().map(|x| f(x)).collect::<Result<Vec<_>,_>>()?;
            EachOrEvery::each(out)
        } else {
            EachOrEvery::every(f(&self.data[0])?)
        })
    }

    pub fn inner_zip<W,F,Y>(&self, other: &EachOrEvery<Y>, cb: F) -> EachOrEvery<W> where F: Fn(&X,&Y) -> W {
        match (&self.index,&other.index) {
            (x,EachOrEveryIndex::Every) => {
                EachOrEvery {
                    index: x.clone(),
                    data: Arc::new(self.data.iter().map(|a| cb(a,&other.data[0])).collect())
                }
            },

            (EachOrEveryIndex::Unindexed, EachOrEveryIndex::Unindexed) => {
                EachOrEvery {
                    index: EachOrEveryIndex::Unindexed,
                    data: Arc::new(self.data.iter().zip(other.data.iter()).map(|(a,b)| cb(a,b)).collect())
                }
            },

            (EachOrEveryIndex::Indexed(index), EachOrEveryIndex::Unindexed) => {
                EachOrEvery {
                    index: EachOrEveryIndex::Unindexed,
                    data: Arc::new(index.iter().zip(other.data.iter()).map(|(a,b)| cb(&self.data[*a],b)).collect())
                }
            },

            (EachOrEveryIndex::Indexed(self_index), EachOrEveryIndex::Indexed(other_index)) => {
                EachOrEvery {
                    index: EachOrEveryIndex::Unindexed,
                    data: Arc::new(self_index.iter().zip(other_index.iter()).map(|(a,b)| cb(&self.data[*a],&other.data[*b])).collect())
                }
            },

            _ => panic!()
        }
    }

    pub fn zip<W,F,Y>(&self, other: &EachOrEvery<Y>, cb: F) -> EachOrEvery<W> where F: Fn(&X,&Y) -> W {
        match (&self.index,&other.index) {
            (EachOrEveryIndex::Every, EachOrEveryIndex::Unindexed) |
            (EachOrEveryIndex::Every, EachOrEveryIndex::Indexed(_)) |
            (EachOrEveryIndex::Unindexed, EachOrEveryIndex::Indexed(_)) => 
                other.inner_zip(self,|a,b| cb(b,a)),

            _ =>
                self.inner_zip(other,cb)
        }
    }

    pub fn iter<'a>(&'a self, len: usize) -> Option<impl Iterator<Item=&'a X>> {
        if let Some(self_len) = self.len() {
            if self_len != len { return None; }
        }
        Some(EachOrEveryIterator {
            obj: self,
            index: 0,
            len
        })
    }

    pub fn make_filter<F>(&self, len: usize, cb: F) -> EachOrEveryFilter where F: Fn(&X) -> bool {
        match &self.index {
            EachOrEveryIndex::Unindexed => {
                let mut filter = EachOrEveryFilterBuilder::new();
                for (i,value) in self.data.iter().enumerate() {
                    if cb(value) {
                        filter.set(i);
                    }
                }
                filter.make(len)
            },
            EachOrEveryIndex::Indexed(index) => {
                let mut filter = EachOrEveryFilterBuilder::new();
                for (i,value) in index.iter().enumerate() {
                    if cb(&self.data[*value]) {
                        filter.set(i);
                    }
                }
                filter.make(len)
            },
            EachOrEveryIndex::Every => {
                if cb(&self.data[0]) {
                    EachOrEveryFilter::all(len)
                } else {
                    EachOrEveryFilter::none(len)
                }
            }
        }
    }

    pub fn filter(&self, data_filter: &EachOrEveryFilter) -> EachOrEvery<X> {
        match &data_filter.data {
            EachOrEveryFilterData::All => self.clone(),
            EachOrEveryFilterData::None => EachOrEvery::each(vec![]),
            EachOrEveryFilterData::Some(filter) => {
                let index = match &self.index {
                    EachOrEveryIndex::Every => EachOrEveryIndex::Every,
                    EachOrEveryIndex::Unindexed => EachOrEveryIndex::Indexed(un_rle(&filter,|i| i)),
                    EachOrEveryIndex::Indexed(index) => EachOrEveryIndex::Indexed(un_rle(&filter,|i| index[i]))
                };
                EachOrEvery { index, data: self.data.clone() }        
            }
        }
    }

    pub fn to_each(&self, len: usize) -> Option<EachOrEvery<X>> {
        match &self.index {
            EachOrEveryIndex::Every => {
                Some(EachOrEvery {
                    index: EachOrEveryIndex::Indexed(Arc::new(vec![0;len])),
                    data: self.data.clone()
                })
            },
            EachOrEveryIndex::Unindexed => {
                if self.data.len() == len { Some(self.clone()) } else { None }
            },
            EachOrEveryIndex::Indexed(index) => {
                if index.len() == len { Some(self.clone()) } else { None }
            }
        }
    }

    pub fn compatible(&self, len: usize) -> bool {
        if let Some(self_len) = self.len() {
            if self_len != len { return false; }
        }
        true
    }
}

pub struct EachOrEveryIterator<'a,X> {
    obj: &'a EachOrEvery<X>,
    index: usize,
    len: usize
}

impl<'a,X> Iterator for EachOrEveryIterator<'a,X> {
    type Item = &'a X;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.len { return None; }
        let out = match &self.obj.index {
            EachOrEveryIndex::Unindexed => &self.obj.data[self.index],
            EachOrEveryIndex::Indexed(index) => &self.obj.data[index[self.index]],
            EachOrEveryIndex::Every => &self.obj.data[0]
        };
        self.index += 1;
        Some(out)
    }
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

#[derive(Clone)]
enum EachOrEveryFilterData {
    All,
    None,
    Some(Vec<(usize,usize)>)
}

#[derive(Clone)]
pub struct EachOrEveryFilter {
    data: EachOrEveryFilterData,
    len: usize,
    count: usize
}

impl EachOrEveryFilter {
    pub fn all(len: usize) -> EachOrEveryFilter {
        return EachOrEveryFilter {
            data: EachOrEveryFilterData::All,
            len, count: len
        };
    }

    pub fn none(len: usize) -> EachOrEveryFilter {
        return EachOrEveryFilter {
            data: EachOrEveryFilterData::None,
            len, count: 0
        };
    }

    pub fn len(&self) -> usize { self.len }
    pub fn count(&self) -> usize { self.count }

    pub fn filter_clone<Z: Clone>(&self, input: &[Z]) -> Vec<Z> {
        match &self.data {
            EachOrEveryFilterData::All => input.to_vec(),
            EachOrEveryFilterData::None => vec![],
            EachOrEveryFilterData::Some(index) => {
                let mut out = vec![];
                for (offset,len) in index {
                    for pos in 0..*len {
                        out.push(input[offset+pos].clone());
                    }
                }
                out
            }
        }
    }

    pub fn and(&self, other: &EachOrEveryFilter) -> EachOrEveryFilter {
        match (&self.data,&other.data) {
            (EachOrEveryFilterData::All,_) => other.clone(),
            (_,EachOrEveryFilterData::All) => self.clone(),
            (EachOrEveryFilterData::None,_) => EachOrEveryFilter::none(self.len),
            (_,EachOrEveryFilterData::None) => EachOrEveryFilter::none(self.len),

            (EachOrEveryFilterData::Some(self_index), EachOrEveryFilterData::Some(other_index)) => {
                intersect(self_index,other_index,self.len)
            }
        }
    }

    pub fn or(&self, other: &EachOrEveryFilter) -> EachOrEveryFilter {
        match (&self.data,&other.data) {
            (EachOrEveryFilterData::All,_) => EachOrEveryFilter::all(self.len()),
            (_,EachOrEveryFilterData::All) => EachOrEveryFilter::all(self.len()),
            (EachOrEveryFilterData::None,_) => other.clone(),
            (_,EachOrEveryFilterData::None) => self.clone(),

            (EachOrEveryFilterData::Some(self_index), EachOrEveryFilterData::Some(other_index)) => {
                union(self_index,other_index,self.len)
            }
        }
    }
}

struct NumIterator<'a> {
    filter: &'a [(usize,usize)],
    range_index: usize,
    pos: usize
}

impl<'a> NumIterator<'a> {
    fn new(filter: &'a [(usize,usize)]) -> NumIterator<'a> {
        NumIterator { filter, range_index: 0, pos: 0 }
    }

    fn peek(&mut self) -> Option<usize> {
        loop {
            if self.range_index >= self.filter.len() { return None; }
            if self.pos < self.filter[self.range_index].1 { break; }
            self.pos = 0;
            self.range_index += 1;
        }
        Some(self.filter[self.range_index].0 + self.pos)
    }

    fn advance(&mut self, index: usize) {
        loop {
            if self.range_index >= self.filter.len() { return; }
            let range = &self.filter[self.range_index];
            if index < range.0 + range.1 {
                self.pos = if index > range.0 { index - range.0 } else { 0 };
                return;
            }
            self.pos = 0;
            self.range_index += 1;
        }
    }
}

fn intersect(a: &[(usize,usize)], b: &[(usize,usize)],len: usize) -> EachOrEveryFilter {
    let mut a_iter = NumIterator::new(a);
    let mut b_iter = NumIterator::new(b);
    let mut out = EachOrEveryFilterBuilder::new();
    loop {
        match (a_iter.peek(),b_iter.peek()) {
            (Some(a),Some(b)) => {
                if a == b { 
                    out.set(a);
                    a_iter.advance(b+1); 
                    b_iter.advance(a+1);
                } else if a < b { 
                    a_iter.advance(b);
                } else if a > b {
                    b_iter.advance(a);
                }
            },
            _ => { break; }
        }
    }
    out.make(len)
}

fn union(a: &[(usize,usize)], b: &[(usize,usize)],len: usize) -> EachOrEveryFilter {
    let mut a_iter = NumIterator::new(a);
    let mut b_iter = NumIterator::new(b);
    let mut out = EachOrEveryFilterBuilder::new();
    loop {
        match (a_iter.peek(),b_iter.peek()) {
            (Some(a),Some(b)) => {
                if a == b { 
                    out.set(a);
                    a_iter.advance(b+1); 
                    b_iter.advance(a+1);
                } else if a < b {
                    out.set(a);
                    a_iter.advance(a+1);
                } else if a > b {
                    out.set(b);
                    b_iter.advance(b+1);
                }
            },
            _ => { break; }
        }
    }
    out.make(len)
}

// XXX run-length
pub struct EachOrEveryFilterBuilder(Vec<(usize,usize)>,usize);

impl EachOrEveryFilterBuilder {
    pub fn new() -> EachOrEveryFilterBuilder { EachOrEveryFilterBuilder(vec![],0) }

    pub fn set(&mut self, index: usize) {
        self.1 += 1;
        if let Some((last_index,last_len)) = self.0.last_mut() {
            if *last_index + *last_len == index {
                *last_len += 1;
                return;
            }
        }
        self.0.push((index,1));
    }

    pub fn make(self, len: usize) -> EachOrEveryFilter {
        if self.0.len() == 0 {
            EachOrEveryFilter::none(len)
        } else {
            if self.0.len() == 1 {
                if self.0[0].0 == 0 && self.0[0].1 == len {
                    return EachOrEveryFilter::all(len);
                }
            }
            EachOrEveryFilter {
                data: EachOrEveryFilterData::Some(self.0),
                len, count: self.1
            }
        }
    }
}
