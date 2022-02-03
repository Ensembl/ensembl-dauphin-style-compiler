use std::collections::BTreeMap;
use std::fmt::Debug;

/* hide some of the horrible inefficeincies of Rust BTrees */

#[derive(Debug)]
enum BoomCursorLocation {
    Start,
    GreaterOrEqual(i64), /* return k or greater */
    Greater(i64),
    End
}

pub struct BoomCursorMut<'a>(&'a mut Boom,BoomCursorLocation);

impl<'a> Debug for BoomCursorMut<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BoomCursorMut").field(&self.1).finish()
    }
}

impl<'a> BoomCursorMut<'a> {
    pub fn tree<'b>(&'b mut self) -> &'b mut Boom { self.0 }

    pub fn rewind(&mut self) -> Option<(i64,f64)> {
        match &self.1 {
            BoomCursorLocation::Start => { None },
            BoomCursorLocation::GreaterOrEqual(cmp) => {
                let mut iter = self.0.tree.range(..cmp).rev();
                if let Some((k,v)) = iter.next() {
                    self.1 = BoomCursorLocation::GreaterOrEqual(k.clone());
                    Some((*k,*v))
                } else {
                    self.1 = BoomCursorLocation::Start;
                    None
                }
            },
            BoomCursorLocation::Greater(cmp) => {
                let mut iter = self.0.tree.range(..(cmp+1)).rev();
                if let Some((k,v)) = iter.next() {
                    self.1 = BoomCursorLocation::GreaterOrEqual(k.clone());
                    Some((*k,*v))
                } else {
                    self.1 = BoomCursorLocation::Start;
                    None
                }                
            }
            BoomCursorLocation::End => {
                if let Some((k,v)) = self.0.tree.range(..).rev().next() {
                    self.1 = BoomCursorLocation::GreaterOrEqual(k.clone());
                    Some((*k,*v))
                } else {
                    self.1 = BoomCursorLocation::Start;
                    None
                }
            }
        }
    }

    pub fn next(&mut self) -> Option<(i64,f64)> {
        match &self.1 {
            BoomCursorLocation::Start => {
                if let Some((k,v)) = self.0.tree.range(..).next() {
                    self.1 = BoomCursorLocation::Greater(k.clone());
                    Some((*k,*v))
                } else {
                    self.1 = BoomCursorLocation::End;
                    None
                }
            },
            BoomCursorLocation::Greater(cmp) => {
                let mut iter = self.0.tree.range((cmp+1)..);
                if let Some((k,v)) = iter.next() {
                    self.1 = BoomCursorLocation::Greater(k.clone());
                    Some((*k,*v))
                } else {
                    self.1 = BoomCursorLocation::End;
                    None
                }
            }
            BoomCursorLocation::GreaterOrEqual(cmp) => {
                let mut iter = self.0.tree.range(cmp..);
                if let Some((k,v)) = iter.next() {
                    self.1 = BoomCursorLocation::Greater(k.clone());
                    Some((*k,*v))
                } else {
                    self.1 = BoomCursorLocation::End;
                    None
                }
            }
            BoomCursorLocation::End => { None }
        }
    }
}

pub struct Boom {
    tree: BTreeMap<i64,f64>
}

impl Boom {
    pub fn new() -> Boom {
        Boom {
            tree: BTreeMap::new()
        }
    }

    pub fn insert(&mut self, key: i64, value: f64) {
        self.tree.insert(key,value);
    }

    pub fn remove(&mut self, key: i64) {
        self.tree.remove(&key);
    }

    pub fn start_mut(&mut self) -> BoomCursorMut {
        BoomCursorMut(self,BoomCursorLocation::Start)
    }

    pub fn seek_mut(&mut self, key: &i64) -> BoomCursorMut {
        if let Some(key) = self.tree.range(key..).next().map(|x| *x.0) {
            BoomCursorMut(self,BoomCursorLocation::GreaterOrEqual(key.clone()))
        } else {
            BoomCursorMut(self,BoomCursorLocation::End)
        }
    }

    pub fn all(&self) -> Vec<(i64,f64)> {
        let mut out = vec![];
        for (k,v) in self.tree.range(..) {
            out.push((*k,*v));
        }
        out
    }
}
