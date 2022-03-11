use std::{hash::Hash, collections::HashMap, sync::Arc};

use crate::EachOrEvery;

pub struct EachOrEveryFilter {
    data: EachOrEvery<bool>,
    len: usize
}

impl<X> EachOrEvery<X> {
    pub fn demerge2<F,K: Hash+PartialEq+Eq>(&self, len: usize, cb: F) -> Vec<(K,EachOrEveryFilter)> where F: Fn(&X) -> K {
        match self {
            EachOrEvery::Every(x) => {
                vec![(cb(x), EachOrEveryFilter {
                    data: EachOrEvery::Every(Arc::new(true)),
                    len
                })]
            },
            EachOrEvery::Each(each) => {
                let mut builders = HashMap::new();
                for (i,x) in each.iter().enumerate() {
                    let k = cb(&x);
                    let entry = builders.entry(k).or_insert_with(|| {
                        (vec![false;len],0)
                    });
                    entry.0[i] = true;
                    entry.1 += 1;
                }
                let mut out = vec![];
                for (key,(filter,len)) in builders.drain() {
                    out.push((key,EachOrEveryFilter {
                        data: EachOrEvery::Each(Arc::new(filter)),
                        len
                    }));
                }
                out
            }
        }
    }
}

impl<X: Clone> EachOrEvery<X> {
    pub fn map_filter(&self, filter: &EachOrEveryFilter) -> EachOrEvery<X> {
        match &filter.data {
            EachOrEvery::Every(x) => {
                if **x { self.clone() } else { EachOrEvery::each(vec![]) }
            },
            EachOrEvery::Each(yn) => {
                match self {
                    EachOrEvery::Every(x) => EachOrEvery::Every(x.clone()),
                    EachOrEvery::Each(x) => {
                        let mut out = vec![];
                        for (value,yn) in x.iter().zip(yn.iter()) {
                            if *yn {
                                out.push(value.clone());
                            }
                        }
                        EachOrEvery::Each(Arc::new(out))
                    }
                }
            }
        }
    }
}