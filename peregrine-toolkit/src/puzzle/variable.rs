use super::{Value, Answer};

pub fn variable<'a,'f: 'a, T: 'a, F: 'f+'a>(f: F) -> Value<'f,'a,T> where F: Fn(&Answer<'a>) -> T {
    Value::new(move |answer_index|
       answer_index.as_ref().map(|answer_index| f(answer_index))
    )
}

#[cfg(test)]
mod test {
    use std::{rc::Rc, cell::RefCell};

    use crate::{puzzle::{AnswerAllocator}};

    use super::variable;

    #[test]
    fn variable_smoke() {
        let mut a = AnswerAllocator::new();
        let x = Rc::new(RefCell::new(2));
        let x2 = x.clone();
        let v = variable(move |_| {
            *x2.borrow() * 12
        });
        assert_eq!(24,v.call(&a.get()));
        *x.borrow_mut() = 3;
        assert_eq!(36,v.call(&a.get()));
    }
}
