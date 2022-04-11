use super::{Value, Answer};

pub fn variable<'a,'f: 'a, T: 'a, F: 'f+'a>(f: F) -> Value<'f,'a,T> where F: Fn(&Answer<'a>) -> T {
    Value::new(move |answer_index|
       answer_index.as_ref().map(|answer_index| f(answer_index))
    )
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use crate::{puzzle3::{AnswerAllocator}, lock};

    use super::variable;

    #[test]
    fn variable_smoke() {
        let mut a = AnswerAllocator::new();
        let x = Arc::new(Mutex::new(2));
        let x2 = x.clone();
        let v = variable(move |_| {
            *lock!(x2) * 12
        });
        assert_eq!(24,v.call(&a.get()));
        *lock!(x) = 3;
        assert_eq!(36,v.call(&a.get()));
    }
}
