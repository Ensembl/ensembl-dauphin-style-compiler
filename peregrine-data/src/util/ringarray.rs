use std::collections::HashMap;
use std::cmp::max;
use std::hash::Hash;

pub struct UniformDataIterator<'a,A: std::fmt::Debug> {
    uniform: &'a UniformData<A>,
    index: usize,
}

impl<'a,A: std::fmt::Debug> Iterator for UniformDataIterator<'a,A> {
    type Item = &'a A;

    fn next(&mut self) -> Option<&'a A> {
        self.index += 1;
        match self.uniform {
            UniformData::None => None,
            UniformData::Uniform(value,size) => {
                if self.index <= *size { Some(value) } else { None }
            }
            UniformData::Varied(values) => {
                if self.index <= values.len() { Some(&values[self.index-1]) } else { None }
            }
        }
    }
}

impl<'a,A: std::fmt::Debug> Clone for UniformDataIterator<'a,A> {
    fn clone(&self) -> UniformDataIterator<'a,A> {
        UniformDataIterator {
            uniform: self.uniform,
            index: 0
        }
    }
}

#[derive(Debug)]
pub enum UniformData<A: std::fmt::Debug> {
    None,
    Uniform(A,usize),
    Varied(Vec<A>)
}

impl<A: Clone+PartialEq+std::fmt::Debug> UniformData<A> {
    pub fn add(&mut self, more: A) {
        match self {
            UniformData::None => { *self = UniformData::Uniform(more,1); },
            UniformData::Uniform(current,count) => {
                if *current == more {
                    *count += 1;
                } else {
                    let mut many = vec![current.clone();*count];
                    many.push(more);
                    *self = UniformData::Varied(many);
                }
            },
            UniformData::Varied(values) => { values.push(more); }
        }
    }

    pub fn get_compact(self) -> Vec<A> {
        match self {
            UniformData::None => vec![],
            UniformData::Uniform(current,_) => vec![current],
            UniformData::Varied(values) => values
        }
    }

    pub fn iter<'a>(&'a self) -> UniformDataIterator<'a,A> {
        UniformDataIterator {
            uniform: &self,
            index: 0
        }
    }

    fn set_size(&mut self, len: usize) {
        match self {
            UniformData::None => {},
            UniformData::Uniform(_,size) => { *size = len },
            UniformData::Varied(values) => {
                if len > values.len() {
                    let orig = values.clone();
                    while len > values.len() {
                        values.append(&mut orig.clone());
                    }
                }
                if len < values.len() {
                    values.truncate(len);
                }
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            UniformData::None => 0,
            UniformData::Uniform(_,size) => *size,
            UniformData::Varied(values) => values.len()
        }
    }

    fn count<F>(&self,mut cb: F) -> usize where F: FnMut(&A) -> bool {
        match self {
            UniformData::None => 0,
            UniformData::Uniform(value,size) => if cb(value) { *size } else { 0 },
            UniformData::Varied(values) => {
                values.iter().map(|v| if cb(v) {1} else {0}).fold(0,|a,b| a+b)
            }
        }
    }
}

pub struct DataFilterBuilder {
    out: DataFilter,
    run_start: Option<usize>,
    most_recent_true: usize
}

impl DataFilterBuilder {
    pub fn new() -> DataFilterBuilder {
        DataFilterBuilder {
            out: DataFilter {
                ranges: vec![],
                size: 0,
                num_set: 0
            },
            run_start: None,
            most_recent_true: 0
        }
    }

    pub fn at(&mut self, index: usize) {
        self.out.num_set += 1;
        if let Some(run_start_at) = self.run_start {
            if index != self.most_recent_true+1 { // prev run ended at self.index
                self.out.ranges.push((run_start_at,self.most_recent_true-run_start_at+1));
                self.run_start = Some(index);
            }
        } else { // first run
            self.run_start = Some(index);
        }
        self.most_recent_true = index;
    }

    pub fn finish(mut self, size: usize) -> DataFilter {
        if let Some(start_at) = self.run_start {
            self.out.ranges.push((start_at,self.most_recent_true-start_at+1));
        }
        self.out.size = size;
        self.out
    }
}

pub struct DataFilterIterator<'a> {
    filter: &'a DataFilter,
    range_index: usize,
    pos: usize
}

impl<'a> DataFilterIterator<'a> {
    fn peek(&mut self) -> Option<usize> {
        loop {
            if self.range_index >= self.filter.ranges.len() { return None; }
            if self.pos < self.filter.ranges[self.range_index].1 { break; }
            self.pos = 0;
            self.range_index += 1;
        }
        Some(self.filter.ranges[self.range_index].0 + self.pos)
    }

    fn advance(&mut self, index: usize) {
        loop {
            if self.range_index >= self.filter.ranges.len() { return; }
            let range = &self.filter.ranges[self.range_index];
            if index < range.0 + range.1 {
                self.pos = if index > range.0 { index - range.0 } else { 0 };
                return;
            }
            self.pos = 0;
            self.range_index += 1;
        }
    }
}

pub struct LoopingDataFilterIterator<'a> {
    filter: &'a DataFilter,
    inner: DataFilterIterator<'a>,
    base: usize,
    limit: usize
}

impl<'a> LoopingDataFilterIterator<'a> {
    fn peek(&mut self) -> Option<usize> {
        if self.filter.size == 0 { return None; }
        loop {
            if let Some(value) = self.inner.peek() {
                if self.base + value >= self.limit { return None; }
                return Some(self.base+value);
            } else {
                if self.base >= self.limit { return None; }
                self.base += self.filter.size;
                self.inner = self.filter.iter();
            }
        }
    }

    fn advance(&mut self, index: usize) {
        if index - self.base >= self.filter.size {
            self.inner = self.filter.iter();
            self.base = (index / self.filter.size) * self.filter.size;
        }
        self.inner.advance(index - self.base);
    }
}

#[derive(Clone)]
pub struct DataFilter {
    ranges: Vec<(usize,usize)>,
    size: usize,
    num_set: usize
}

impl DataFilter {
    pub fn new<F,X>(data: &mut dyn Iterator<Item=X>, cb: F) -> DataFilter where F: Fn(X) -> bool {
        let mut builder = DataFilterBuilder::new();
        let mut count = 0;
        for (i,item) in data.enumerate() {
            if cb(item) {
                builder.at(i);
            }
            count += 1;
        }
        builder.finish(count)
    }

    pub fn iter<'a>(&'a self) -> DataFilterIterator<'a> {
        DataFilterIterator {
            filter: &self,
            range_index: 0,
            pos: 0
        }
    }

    pub fn iter_num<'a>(&'a self, len: usize) -> LoopingDataFilterIterator<'a> {
        LoopingDataFilterIterator {
            filter: &self,
            inner: self.iter(),
            base: 0,
            limit: len
        }
    }

    fn double_to(&mut self, size: usize) {
        let orig_range_len = self.ranges.len();
        for i in 0..orig_range_len {
            let mut range = (self.ranges[i].0 + self.size, self.ranges[i].1);
            if range.0 >= size { break; }
            if range.0 + range.1 > size { range.1 = size - range.0; }
            self.num_set += range.1;
            self.ranges.push(range);
        }
    }

    fn chop_down(&mut self, size: usize) {
        self.num_set = 0;
        let mut new_range_len = 0;
        for (start,length) in &mut self.ranges {
            if *start >= size { break; }
            new_range_len += 1;
            if *start+*length > size {
                *length = size-*start;
                self.num_set += *length;
                break;
            } else {
                self.num_set += *length;
            }
        }
        self.ranges.truncate(new_range_len);
    }

    pub fn set_size(&mut self, size: usize) {
        if self.num_set == self.size {
            self.ranges[0].1 = size;
            self.num_set = size;
        } else if self.num_set != 0 {
            while self.size < size {
                self.double_to(size);
            }
            if self.size > size {
                self.chop_down(size);
            }
        }
        self.size = size;
    }

    pub fn and(&self, other: &DataFilter) -> DataFilter {
        let len = max(self.len(),other.len());
        let mut a_iter = self.iter_num(len);
        let mut b_iter = other.iter_num(len);
        let mut out = DataFilterBuilder::new();
        loop {
            match (a_iter.peek(),b_iter.peek()) {
                (Some(a),Some(b)) => {
                    if a == b { 
                        out.at(a);
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
        out.finish(len)
    }

    pub fn demerge<F,X,K: Hash+PartialEq+Eq>(data: &[X],cb: F) -> Vec<(K,DataFilter)> where F: Fn(&X) -> K {
        let mut builders = HashMap::new();
        let mut size = 0;
        for (i,value) in data.iter().enumerate() {
            size += 1;
            let kind = cb(value);
            builders.entry(kind).or_insert_with(|| DataFilterBuilder::new()).at(i);
        }
        builders.drain().map(|(k,v)| (k,v.finish(size))).collect()
    }

    /* VERY HOT CODE PATH: PREFER SPEED OVER ELEGANCE */
    pub fn filter<X: Clone>(&self, other: &[X]) -> Vec<X> {
        let other_full_len = other.len();
        let mut out = Vec::with_capacity(self.num_set);
        for (start,len) in &self.ranges {
            let mut other_start = *start % other_full_len;
            let mut other_len = *len;
            while other_start + other_len >= other_full_len {
                out.extend_from_slice(&other[other_start..]);
                other_len -= other_full_len - other_start;
                other_start = 0;
            }
            out.extend_from_slice(&other[other_start..(other_start+other_len)]);
        }
        out
    }

    pub fn len(&self) -> usize { self.size }
    pub fn count(&self) -> usize { self.num_set }
}
