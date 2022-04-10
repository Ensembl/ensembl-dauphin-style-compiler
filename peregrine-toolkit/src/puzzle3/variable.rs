use super::{Solver, AnswerIndex};

pub fn variable<'a,'f: 'a, T: 'a, F: 'f+'a>(f: F) -> Solver<'f,'a,T> where F: Fn(&AnswerIndex<'a>) -> T {
    Solver::new(move |answer_index|
       answer_index.as_ref().map(|answer_index| f(answer_index))
    )
}
