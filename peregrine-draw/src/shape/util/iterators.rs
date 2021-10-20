use crate::Message;

pub struct IterFixed<T,U> where T: Iterator<Item=U> {
    iter: T,
    remaining: usize
}

impl<T,U> IterFixed<T,U> where T: Iterator<Item=U> {
    pub(super) fn new(iter: T, repeat: usize) -> IterFixed<T,U> {
        IterFixed { iter, remaining: repeat }
    }
}

impl<T,U> Iterator for IterFixed<T,U> where T: Iterator<Item=U> {
    type Item = U;

    fn next(&mut self) -> Option<U> {
        if self.remaining <= 0 { return None; }
        self.remaining -= 1;
        self.iter.next()
    }
}

pub struct IterInterleave<T,U> where T: Iterator<Item=U> {
    iters: Vec<T>,
    offset: usize
}

impl<T,U> IterInterleave<T,U> where T: Iterator<Item=U> {
    pub(super) fn new(iters: Vec<T>) -> IterInterleave<T,U> {
        IterInterleave { iters, offset: 0 }
    }
}

impl<T,U> Iterator for IterInterleave<T,U> where T: Iterator<Item=U> {
    type Item = U;

    fn next(&mut self) -> Option<U> {
        let out = self.iters[self.offset].next();
        self.offset += 1;
        self.offset %= self.iters.len();
        out
    }
}

#[derive(Clone)]
pub struct IterRepeat<T,U> where T: Iterator<Item=U>, U: Clone {
    iter: T,
    latest: Option<U>,
    target: usize,
    so_far: usize
}

impl<T,U> IterRepeat<T,U> where T: Iterator<Item=U>, U: Clone {
    pub(super) fn new(iter: T, count: usize) -> IterRepeat<T,U> {
        IterRepeat {
            iter,
            latest: None,
            target: count,
            so_far: count
        }
    }
}

impl <T,U> Iterator for IterRepeat<T,U> where T: Iterator<Item=U>, U: Clone {
    type Item = U;

    fn next(&mut self) -> Option<U> {
        if self.target == self.so_far {
            self.latest = self.iter.next();
            self.so_far = 0;
        }
        self.latest.clone()
    }
}

pub fn eoe_throw<X>(kind: &str,input: Option<X>) -> Result<X,Message> {
    input.ok_or_else(|| Message::LengthMismatch(kind.to_string()))
}
