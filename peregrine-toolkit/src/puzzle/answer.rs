use std::{collections::BTreeSet, rc::{Rc, Weak}, cell::RefCell};
use crate::identitynumber;

identitynumber!(ANSWER_IDS);

struct OpaqueRcHolder<T>(Rc<T>);
trait OpaqueRcTrait {}
impl<T> OpaqueRcTrait for OpaqueRcHolder<T> {}

struct OpaqueRc<'a>(Box<dyn OpaqueRcTrait + 'a>);

impl<'a> OpaqueRc<'a> {
    fn new<T: 'a>(rc: Rc<T>) -> OpaqueRc<'a> {
        OpaqueRc(Box::new(OpaqueRcHolder(rc)))
    }
}

pub struct Answer<'a> {
    serial: u64,
    index: usize,
    allocator: AnswerAllocator,
    retained: Rc<RefCell<Vec<OpaqueRc<'a>>>>
}

pub type StaticAnswer = Answer<'static>;

impl<'a> Answer<'a> {
    pub fn index(&self) -> usize { self.index }
    pub fn serial(&self) -> u64 { self.serial }

    pub fn retain<T: 'a>(&self, input: &Rc<T>) -> Weak<T> {
        self.retained.borrow_mut().push(OpaqueRc::new(input.clone()));
        let output = Rc::downgrade(input);
        output
    }
}

impl<'a> Drop for Answer<'a> {
    fn drop(&mut self) {
       self.allocator.0.borrow_mut().put_answer_index(self.index);
    }
}

struct AnswerAllocatorData {
    next_never_used_answer: usize,
    recycled_answers: BTreeSet<usize>
}

impl AnswerAllocatorData {
    fn new() -> AnswerAllocatorData {
        AnswerAllocatorData {
            next_never_used_answer: 0,
            recycled_answers: BTreeSet::new()
        }
    }

    fn get(&mut self) -> usize {
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
pub struct AnswerAllocator(Rc<RefCell<AnswerAllocatorData>>);

impl AnswerAllocator {
    pub fn new() -> AnswerAllocator {
        AnswerAllocator(Rc::new(RefCell::new(AnswerAllocatorData::new())))
    }

    pub fn get<'a>(&mut self) -> Answer<'a> {
        Answer {
            serial: ANSWER_IDS.next(),
            index: self.0.borrow_mut().get(),
            allocator: self.clone(),
            retained: Rc::new(RefCell::new(vec![]))
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use super::AnswerAllocator;

    #[test]
    fn answer_smoke() {
        let mut allocator = AnswerAllocator::new();
        let mut serials = HashSet::new();
        let a0a = allocator.get();
        assert_eq!(0,a0a.index());
        serials.insert(a0a.serial());
        drop(a0a);
        let a0b = allocator.get();
        assert_eq!(0,a0b.index());
        serials.insert(a0b.serial());
        let a1a = allocator.get();
        assert_eq!(1,a1a.index());
        serials.insert(a1a.serial());
        drop(a0b);
        let a0c = allocator.get();
        assert_eq!(0,a0c.index());
        serials.insert(a0c.serial());
        let a2a = allocator.get();
        assert_eq!(2,a2a.index());
        serials.insert(a2a.serial());
        let a3a = allocator.get();
        assert_eq!(3,a3a.index());
        serials.insert(a3a.serial());
        drop(a0c);
        drop(a2a);
        drop(a3a);
        let a0d = allocator.get();
        assert_eq!(0,a0d.index());
        serials.insert(a0d.serial());
        let a2b = allocator.get();
        assert_eq!(2,a2b.index());
        serials.insert(a2b.serial());
        let a3b = allocator.get();
        assert_eq!(3,a3b.index());
        serials.insert(a3b.serial());
        let a4a = allocator.get();
        assert_eq!(4,a4a.index());
        serials.insert(a4a.serial());
        drop(a0d);
        drop(a1a);
        drop(a2b);
        drop(a3b);
        drop(a4a);
        assert_eq!(10,serials.len());
    }

    #[test]
    fn serial_sequence() {
        let mut a1 = AnswerAllocator::new();
        let mut a2 = AnswerAllocator::new();
        let mut prev_serial = 0;
        a1.get(); // ensure at least 1
        for i in 0..100 {
            let ai = if i%2==0 { &mut a1 } else { &mut a2 };
            let a = ai.get();
            assert!(a.serial()>prev_serial);
            prev_serial = a.serial();
        }
    }
}
