use std::{collections::BTreeSet, sync::{Arc, Mutex, Weak}};
use crate::lock;

use lazy_static::lazy_static;
use identitynumber::identitynumber;

identitynumber!(IDS);

struct OpaqueArcHolder<T>(Arc<T>);
trait OpaqueArcTrait {}
impl<T> OpaqueArcTrait for OpaqueArcHolder<T> {}

pub struct OpaqueArc<'a>(Box<dyn OpaqueArcTrait + 'a>);

impl<'a> OpaqueArc<'a> {
    fn new<T: 'a>(arc: Arc<T>) -> OpaqueArc<'a> {
        OpaqueArc(Box::new(OpaqueArcHolder(arc)))
    }
}

pub struct AnswerIndex<'a> {
    serial: u64,
    index: usize,
    allocator: AnswerIndexAllocator,
    retained: Arc<Mutex<Vec<OpaqueArc<'a>>>>
}

impl<'a> AnswerIndex<'a> {
    pub fn index(&self) -> usize { self.index }
    pub fn serial(&self) -> u64 { self.serial }

    pub fn retain<T: 'a>(&self, input: &Arc<T>) -> Weak<T> {
        lock!(self.retained).push(OpaqueArc::new(input.clone()));
        let output = Arc::downgrade(input);
        output
    }
}

impl<'a> Drop for AnswerIndex<'a> {
    fn drop(&mut self) {
       lock!(self.allocator.0).put_answer_index(self.index);
    }
}

struct AnswerIndexAllocatorData {
    next_never_used_answer: usize,
    recycled_answers: BTreeSet<usize>
}

impl AnswerIndexAllocatorData {
    fn new() -> AnswerIndexAllocatorData {
        AnswerIndexAllocatorData {
            next_never_used_answer: 0,
            recycled_answers: BTreeSet::new()
        }
    }

    fn get_answer_index(&mut self) -> usize {
        if let Some(value) = self.recycled_answers.iter().next().cloned() {
            self.recycled_answers.remove(&value);
           value
        } else {
            self.next_never_used_answer += 1;
            self.next_never_used_answer-1
        }
    }

    fn put_answer_index(&mut self, index: usize) {
        self.recycled_answers.insert(index);
    }
}

#[derive(Clone)]
pub struct AnswerIndexAllocator(Arc<Mutex<AnswerIndexAllocatorData>>);

impl AnswerIndexAllocator {
    pub fn new() -> AnswerIndexAllocator {
        AnswerIndexAllocator(Arc::new(Mutex::new(AnswerIndexAllocatorData::new())))
    }

    pub fn get_answer_index<'a>(&mut self) -> AnswerIndex<'a> {
        AnswerIndex {
            serial: IDS.next(),
            index: lock!(self.0).get_answer_index(),
            allocator: self.clone(),
            retained: Arc::new(Mutex::new(vec![]))
        }
    }
}
