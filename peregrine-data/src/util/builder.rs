use std::sync::{ Arc, Mutex };
use owning_ref::MutexGuardRefMut;

pub struct Builder<T>(Arc<Mutex<Option<T>>>);

impl<T> Clone for Builder<T> {
    fn clone(&self) -> Self {
        Builder(self.0.clone())
    }
}

impl<T> Builder<T> {
    pub fn new(new: T) -> Builder<T> {
        Builder(Arc::new(Mutex::new(Some(new))))
    }

    pub fn build(self) -> T {
        self.0.lock().unwrap().take().unwrap()
    }

    pub fn lock(&self) -> MutexGuardRefMut<Option<T>,T> {
        MutexGuardRefMut::new(self.0.lock().unwrap()).map_mut(|x| x.as_mut().unwrap())
    }
}
