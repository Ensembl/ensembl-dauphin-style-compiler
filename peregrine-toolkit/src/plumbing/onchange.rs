use std::sync::{Arc, Mutex};

use crate::lock;

pub struct OnChange<T: PartialEq>(Option<T>);

impl<T: PartialEq> OnChange<T> {
    pub fn new() -> OnChange<T> {
        OnChange(None)
    }

    pub fn update<F>(&mut self, value: T, cb: F) -> bool where F: FnOnce(&T) {
        if let Some(old_value) = &self.0 {
            if old_value == &value {
                return false;
            }
        }
        cb(&value);
        self.0 = Some(value);
        true
    }
}

pub struct MutexOnChange<T: PartialEq>(Mutex<Arc<Option<T>>>);

impl<T: PartialEq> MutexOnChange<T> {
    pub fn new() -> MutexOnChange<T> {
        MutexOnChange(Mutex::new(Arc::new(None)))
    }

    pub fn update<F>(&self, value: T, cb: F) -> bool where F: FnOnce(&T) {
        if let Some(old_value) = lock!(self.0).as_ref() {
            if old_value == &value {
                return false;
            }
        }
        let new = Arc::new(Some(value));
        *lock!(self.0) = new.clone();
        cb(new.as_ref().as_ref().unwrap());
        true
    }
}
