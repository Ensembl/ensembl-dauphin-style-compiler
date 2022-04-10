use std::sync::Arc;

use super::answer::AnswerIndex;

pub trait Store<'a, T> {
    fn set<'b>(&mut self, answer_index: &AnswerIndex<'a>, value: Arc<T>);
    fn get(&self, index: &AnswerIndex<'a>) -> Option<Arc<T>>;
}
