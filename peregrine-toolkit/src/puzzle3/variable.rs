use super::{Value, Answer};

pub fn variable<'a,'f: 'a, T: 'a, F: 'f+'a>(f: F) -> Value<'f,'a,T> where F: Fn(&Answer<'a>) -> T {
    Value::new(move |answer_index|
       answer_index.as_ref().map(|answer_index| f(answer_index))
    )
}
