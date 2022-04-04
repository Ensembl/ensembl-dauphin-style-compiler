use std::sync::{Arc, Mutex};

use crate::lock;

#[derive(Clone)]
pub(super) struct AnswerIndex(usize);

pub(super) struct Answers<T> {
    values: Arc<Mutex<Vec<Option<Arc<T>>>>>,

    #[cfg(any(debug_assertions,test))]
    bid: u64
}

// Rust bug means dan't derive Clone on polymorphic types
impl<T> Clone for Answers<T> {
    fn clone(&self) -> Self {
        Answers {
            values: self.values.clone(),

            #[cfg(any(debug_assertions,test))]
            bid: self.bid.clone()
        }
    }
}

impl<T> Answers<T> {
    pub(super) fn new(bid: u64) -> Answers<T> {
        Answers {
            values:  Arc::new(Mutex::new(vec![])),

            #[cfg(any(debug_assertions,test))]
            bid
        }
    }

    #[cfg(any(debug_assertions,test))]
    pub(super) fn check_for_aliens(&self, bid: u64, name: &str) {
        if bid != self.bid {
            panic!("alien piece {}: piece for puzzle {}, solution for puzzle {}",name,self.bid,bid);
        }
    }

    pub(super) fn get(&self, index: &AnswerIndex) -> Option<Arc<T>> {
        lock!(self.values)[index.0].clone()
    }

    pub(super) fn set(&self, value: T) -> AnswerIndex {
        let mut values = lock!(self.values);
        let mut index = None;
        for (i,v) in values.iter_mut().enumerate() {
            if v.is_none() {
                index = Some(i);
                break;
            }
        }
        let index = match index {
            Some(index) => index,
            None => {
                values.push(None);
                values.len()-1
            }
        };
        values[index] = Some(Arc::new(value));
        AnswerIndex(index)
    }

    pub(super) fn finish(&self, index: &AnswerIndex) {
        let mut values = lock!(self.values);
        values[index.0] = None;
        while let Some(&None) = values.last() {
            values.pop();
        }
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        lock!(self.values).len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn to_value<T: Clone>(value: Option<Arc<T>>) -> Option<T> {
        value.map(|x| x.as_ref().clone())
    }

    #[test]
    fn answer_smoke() {
        let ans = Answers::new(0);   
        let a1 = ans.set(1);
        let a2 = ans.set(2);
        assert_eq!(Some(1),to_value(ans.get(&a1)));
        assert_eq!(Some(2),to_value(ans.get(&a2)));
        assert_eq!(2,ans.len());
        ans.finish(&a1);
        assert_eq!(None,to_value(ans.get(&a1)));
        assert_eq!(Some(2),to_value(ans.get(&a2)));
        assert_eq!(2,ans.len());
        ans.finish(&a2);
        assert_eq!(0,ans.len());
        let a3 = ans.set(3);
        let a4 = ans.set(4);
        let a5 = ans.set(5);
        assert_eq!(Some(3),to_value(ans.get(&a3)));
        assert_eq!(Some(4),to_value(ans.get(&a4)));
        assert_eq!(Some(5),to_value(ans.get(&a5)));
        assert_eq!(3,ans.len());
        ans.finish(&a5);
        assert_eq!(Some(3),to_value(ans.get(&a3)));
        assert_eq!(Some(4),to_value(ans.get(&a4)));
        assert_eq!(2,ans.len());
        let a6 = ans.set(6);
        assert_eq!(Some(6),to_value(ans.get(&a6)));
        assert_eq!(3,ans.len());
        assert_eq!(Some(3),to_value(ans.get(&a3)));
        assert_eq!(Some(4),to_value(ans.get(&a4)));
        assert_eq!(Some(6),to_value(ans.get(&a6)));
        ans.finish(&a4);
        assert_eq!(3,ans.len());
        assert_eq!(Some(3),to_value(ans.get(&a3)));
        assert_eq!(Some(6),to_value(ans.get(&a6)));
        ans.finish(&a6);
        assert_eq!(1,ans.len());
        assert_eq!(Some(3),to_value(ans.get(&a3)));
        ans.finish(&a3);
        assert_eq!(0,ans.len());
    }
}
