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