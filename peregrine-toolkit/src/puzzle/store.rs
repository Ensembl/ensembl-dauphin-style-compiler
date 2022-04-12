use std::sync::Arc;

use super::answer::Answer;

pub trait Store<'a, T> {
    fn set<'b>(&mut self, answer_index: &Answer<'a>, value: Arc<T>);
    fn get(&self, index: &Answer<'a>) -> Option<Arc<T>>;
}
