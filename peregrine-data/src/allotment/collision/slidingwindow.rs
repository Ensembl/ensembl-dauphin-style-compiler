use std::collections::VecDeque;

pub(super) enum SlidingWindowContext<'a,T> {
    Fresh(&'a mut T),
    Left(Option<&'a T>,&'a mut T),
    Right(&'a mut T,Option<&'a T>)
}

pub(super) struct SlidingWindow<'a,T> {
    length: usize,
    left: Option<usize>,
    index: Box<dyn (Fn(&T) -> usize) + 'a>,
    make: Box<dyn Fn(SlidingWindowContext<'_,T>) + 'a>,
    remove: Box<dyn Fn(&mut T) + 'a>,
    stores: VecDeque<T> // "back" is left, "front" is right.
}

impl<'a,T> SlidingWindow<'a,T> {
    pub(super) fn new<F,G,H>(length: usize, index: H, make: F, remove: G) -> SlidingWindow<'a,T>
            where F: Fn(SlidingWindowContext<'_,T>) + 'a, G: Fn(&mut T) + 'a, H: Fn(&T) -> usize + 'a {
        SlidingWindow {
            length,
            left: None,
            index: Box::new(index),
            make: Box::new(make),
            remove: Box::new(remove),
            stores: VecDeque::new()
        }
    }

    fn add_first(&mut self, index: usize, mut store: T) {
        (self.make)(SlidingWindowContext::Fresh(&mut store));
        self.left = Some(index);
        self.stores.push_front(store);
        if self.stores.len() > self.length {
            let mut gone = self.stores.pop_back().unwrap();
            (self.remove)(&mut gone);
            *self.left.as_mut().unwrap() += 1;
        }
    }

    fn add_left(&mut self, mut store: T) {
        (self.make)(SlidingWindowContext::Left(None,&mut store));
        *self.left.as_mut().unwrap() -= 1;
        self.stores.push_back(store);
        if self.stores.len() > self.length {
            let mut gone = self.stores.pop_front().unwrap();
            (self.remove)(&mut gone);
        }
    }

    fn add_right(&mut self, mut store: T) {
        (self.make)(SlidingWindowContext::Right(&mut store,None));
        self.stores.push_front(store);
    }

    pub(super) fn add(&mut self, store: T) -> bool {
        let index = (self.index)(&store);
        if let Some(left) = self.left {
            if index == left-1 {
                self.add_left(store);
                true
            } else if index == left + self.stores.len() {
                self.add_right(store);
                true
            } else {
                false
            }
        } else {
            self.add_first(index,store);
            true
        }
    }
}
