use std::sync::{Arc, Weak};

#[derive(Clone)]
pub struct RetainTest(Weak<()>);

impl RetainTest {
    pub fn test(&self) -> bool { Weak::upgrade(&self.0).is_some() }
}

#[derive(Clone)]
pub struct Retainer(Arc<()>);

impl Retainer {
    pub fn test(&self) -> RetainTest { RetainTest(Arc::downgrade(&self.0)) }
}

pub fn retainer() -> (Retainer,RetainTest) {
    let strong = Retainer(Arc::new(()));
    (strong.clone(),strong.test())
}
