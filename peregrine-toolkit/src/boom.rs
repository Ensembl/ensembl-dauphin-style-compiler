use std::collections::BTreeMap;
use std::fmt::Debug;

/* Boom is a BTree(i64->T) implementation which wraps Rust's native BTree implementation to hide some
 * horrible inefficiencies. As well as the usual operations (insert, remove, all), there is an unusual 
 * pseudo-iterator, BoomCursorMut. As well as a next() method, it has rewind() which efficiently goes to the
 * entry *before* the current position. One of these iterators can be placed at the start() or at some key.
 * All these methods are present in Rust's btreemap but are horribly inefficient.
 */

#[derive(Debug)]
enum BoomCursorLocation {
    Start,
    GreaterOrEqual(i64), /* return k or greater */
    Greater(i64),
    End
}

pub struct BoomCursorMut<'a,T>(&'a mut Boom<T>,BoomCursorLocation);

impl<'a,T> Debug for BoomCursorMut<'a,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BoomCursorMut").field(&self.1).finish()
    }
}

impl<'a,T> BoomCursorMut<'a,T> {
    pub fn tree<'b>(&'b mut self) -> &'b mut Boom<T> { self.0 }

    pub fn rewind(&mut self) -> Option<(i64,&T)> {
        match &self.1 {
            BoomCursorLocation::Start => { None },
            BoomCursorLocation::GreaterOrEqual(cmp) => {
                let mut iter = self.0.tree.range(..cmp).rev();
                if let Some((k,v)) = iter.next() {
                    self.1 = BoomCursorLocation::GreaterOrEqual(k.clone());
                    Some((*k,v))
                } else {
                    self.1 = BoomCursorLocation::Start;
                    None
                }
            },
            BoomCursorLocation::Greater(cmp) => {
                let mut iter = self.0.tree.range(..(cmp+1)).rev();
                if let Some((k,v)) = iter.next() {
                    self.1 = BoomCursorLocation::GreaterOrEqual(k.clone());
                    Some((*k,v))
                } else {
                    self.1 = BoomCursorLocation::Start;
                    None
                }                
            }
            BoomCursorLocation::End => {
                if let Some((k,v)) = self.0.tree.range(..).rev().next() {
                    self.1 = BoomCursorLocation::GreaterOrEqual(k.clone());
                    Some((*k,v))
                } else {
                    self.1 = BoomCursorLocation::Start;
                    None
                }
            }
        }
    }

    pub fn next(&mut self) -> Option<(i64,&T)> {
        match &self.1 {
            BoomCursorLocation::Start => {
                if let Some((k,v)) = self.0.tree.range(..).next() {
                    self.1 = BoomCursorLocation::Greater(k.clone());
                    Some((*k,v))
                } else {
                    self.1 = BoomCursorLocation::End;
                    None
                }
            },
            BoomCursorLocation::Greater(cmp) => {
                let mut iter = self.0.tree.range((cmp+1)..);
                if let Some((k,v)) = iter.next() {
                    self.1 = BoomCursorLocation::Greater(k.clone());
                    Some((*k,v))
                } else {
                    self.1 = BoomCursorLocation::End;
                    None
                }
            }
            BoomCursorLocation::GreaterOrEqual(cmp) => {
                let mut iter = self.0.tree.range(cmp..);
                if let Some((k,v)) = iter.next() {
                    self.1 = BoomCursorLocation::Greater(k.clone());
                    Some((*k,v))
                } else {
                    self.1 = BoomCursorLocation::End;
                    None
                }
            }
            BoomCursorLocation::End => { None }
        }
    }
}

pub struct Boom<T> {
    tree: BTreeMap<i64,T>
}

impl<T> Boom<T> {
    pub fn new() -> Boom<T> {
        Boom {
            tree: BTreeMap::new()
        }
    }

    pub fn insert(&mut self, key: i64, value: T) {
        self.tree.insert(key,value);
    }

    pub fn remove(&mut self, key: i64) {
        self.tree.remove(&key);
    }

    pub fn start_mut(&mut self) -> BoomCursorMut<T> {
        BoomCursorMut(self,BoomCursorLocation::Start)
    }

    pub fn seek_mut(&mut self, key: &i64) -> BoomCursorMut<T> {
        if let Some(key) = self.tree.range(key..).next().map(|x| *x.0) {
            BoomCursorMut(self,BoomCursorLocation::GreaterOrEqual(key.clone()))
        } else {
            BoomCursorMut(self,BoomCursorLocation::End)
        }
    }

    pub fn all(&self) -> Vec<(i64,&T)> {
        let mut out = vec![];
        for (k,v) in self.tree.range(..) {
            out.push((*k,v));
        }
        out
    }
}
