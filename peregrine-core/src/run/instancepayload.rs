use std::any::Any;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use dauphin_interp::{ Payload, PayloadFactory };
use owning_ref::MutexGuardRef;

#[derive(Clone)]
pub struct InstancePayload(HashMap<String,Arc<Mutex<Box<dyn Any>>>>);

impl InstancePayload {
    pub fn new(mut instance: HashMap<String,Box<dyn Any>>) -> InstancePayload {
        InstancePayload(instance.drain().map(|(k,v)| (k.to_string(),Arc::new(Mutex::new(v)))).collect())
    }

    pub fn get<'a>(&'a self, key: &'a str) -> Option<MutexGuardRef<Box<dyn Any>>> {
        self.0.get(key).map(|x| MutexGuardRef::new(x.lock().unwrap()))
    }
}

impl Payload for InstancePayload {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn finish(&mut self) {}
}

impl PayloadFactory for InstancePayload {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(self.clone())
    }
}
