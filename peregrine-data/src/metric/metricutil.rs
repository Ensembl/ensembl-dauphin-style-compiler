use std::hash::Hash;
use std::collections::HashMap;

pub(super) struct FactoredValueBuilder<T> {
    lookup: HashMap<T,usize>
}

impl<T> FactoredValueBuilder<T> where T: Eq+Hash+Clone {
    pub(super) fn new() -> FactoredValueBuilder<T> {
        FactoredValueBuilder {
            lookup: HashMap::new()
        }
    }

    pub(super) fn lookup(&mut self, value: &T) -> usize {
        let len = self.lookup.len();
        *self.lookup.entry(value.clone()).or_insert(len)
    }

    pub(super) fn build(&mut self) -> Vec<T> {
        let (placeholder,_) = if let Some(placeholder) = self.lookup.iter().next() {
            placeholder
        } else {
            return vec![]
        };
        let mut out = vec![placeholder.clone();self.lookup.iter().map(|x| *x.1+1).max().unwrap_or(0)];
        for (t,index) in self.lookup.drain() {
            out[index] = t.clone();
        }
        out
    }
}
