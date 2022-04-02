use std::collections::{VecDeque, HashSet};

pub(super) enum SlidingWindowContext<'a,T> {
    Fresh(&'a mut T),
    Left(&'a mut T,&'a T),
    Right(&'a T,&'a mut T)
}

pub(super) struct SlidingWindow<'a,T,U> {
    length: usize,
    left: Option<usize>,
    locked: HashSet<usize>,
    index: Box<dyn (Fn(&T) -> usize) + 'a>,
    make: Box<dyn Fn(SlidingWindowContext<'_,T>) -> U + 'a>,
    remove: Box<dyn Fn(&mut T) + 'a>,
    stores: VecDeque<T> // "back" is left, "front" is right.
}

impl<'a,T,U> SlidingWindow<'a,T,U> {
    pub(super) fn new<F,G,H>(length: usize, index: H, make: F, remove: G) -> SlidingWindow<'a,T,U>
            where F: Fn(SlidingWindowContext<'_,T>) -> U + 'a, G: Fn(&mut T) + 'a, H: Fn(&T) -> usize + 'a {
        SlidingWindow {
            length,
            left: None,
            locked: HashSet::new(),
            index: Box::new(index),
            make: Box::new(make),
            remove: Box::new(remove),
            stores: VecDeque::new()
        }
    }

    fn add_first(&mut self, index: usize, mut store: T) -> U {
        let out= (self.make)(SlidingWindowContext::Fresh(&mut store));
        self.left = Some(index);
        self.stores.push_front(store);
        out
    }

    fn add_left(&mut self, mut store: T) -> U {
        let out = (self.make)(SlidingWindowContext::Left(&mut store,self.stores.back().unwrap()));
        *self.left.as_mut().unwrap() -= 1;
        self.stores.push_back(store);
        /* remove from right if necessary/possible */
        let rightmost = self.left.unwrap() + self.stores.len() - 1;
        if self.stores.len() > self.length && !self.locked.contains(&rightmost) {
            let mut gone = self.stores.pop_front().unwrap();
            (self.remove)(&mut gone);
        }
        out
    }

    fn add_right(&mut self, mut store: T) -> U {
        let out = (self.make)(SlidingWindowContext::Right(self.stores.front().unwrap(),&mut store));
        self.stores.push_front(store);
        /* remove from right if necessary/possible */
        let leftmost = self.left.unwrap();
        if self.stores.len() > self.length && !self.locked.contains(&leftmost) {
            let mut gone = self.stores.pop_back().unwrap();
            (self.remove)(&mut gone);
            *self.left.as_mut().unwrap() += 1;
        }
        out
    }

    pub(super) fn add(&mut self, store: T) -> Option<U> {
        let index = (self.index)(&store);
        if let Some(left) = self.left {
            if index == left-1 {
                Some(self.add_left(store))
            } else if index == left + self.stores.len() {
                Some(self.add_right(store))
            } else {
                None
            }
        } else {
            Some(self.add_first(index,store))
        }
    }

    pub(super) fn set_lock(&mut self, index: usize, yn: bool) {
        let left = if let Some(left) = self.left { left } else { return; };
        if index < left || index >= self.stores.len() + left { return; }
        if yn {
            self.locked.insert(index);
        } else {
            self.locked.remove(&index);
        }
    }

    pub(super) fn get(&self, index: usize) -> Option<&T> {
        let left = if let Some(left) = self.left { left } else { return None; };
        if index < left || index >= self.stores.len() + left { return None; }
        self.stores.get(index - left)
    }
}

impl<'a,T,U> Drop for SlidingWindow<'a,T,U> {
    fn drop(&mut self) {
        for mut source in self.stores.drain(..) {
            (self.remove)(&mut source);
        }
    }
}
