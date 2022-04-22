use std::sync::{Arc, Weak};

#[derive(Clone)]
pub struct RetainTest(Weak<()>);

impl RetainTest {
    pub fn test(&self) -> bool { Weak::upgrade(&self.0).is_some() }
}

#[derive(Clone)]
pub struct Retainer(Arc<()>);

pub fn retainer() -> (Retainer,RetainTest) {
    let strong = Arc::new(());
    let weak = Arc::downgrade(&strong);
    (Retainer(strong),RetainTest(weak))
}
