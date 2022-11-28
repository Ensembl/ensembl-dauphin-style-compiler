use std::rc::Rc;

use super::answer::Answer;

pub trait Store<'a, T> {
    fn set<'b>(&mut self, answer_index: &Answer<'a>, value: Rc<T>);
    fn get(&self, index: &Answer<'a>) -> Option<Rc<T>>;
}
