use std::{sync::{Mutex, Arc}, collections::HashMap, rc::Rc};

use commander::CommanderStream;
use peregrine_toolkit::lock;

use super::{response::{BackendResponse, BackendResponseAttempt}, request::{MiniRequest, MiniRequestAttempt}};

#[derive(Clone)]
pub(crate) struct AttemptMatch {
    pending: Arc<Mutex<HashMap<u64,CommanderStream<BackendResponse>>>>,
    next_id: Arc<Mutex<u64>>
}

impl AttemptMatch {
    pub(super) fn new() -> AttemptMatch {
        AttemptMatch {
            pending: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0))
        }
    }

    fn next_id(&self) -> u64 {
        let mut id = lock!(self.next_id);
        *id += 1;
        *id
    }

    fn add_pending(&self, id: u64, response: &CommanderStream<BackendResponse>) {
        let mut pending = lock!(self.pending);
        pending.insert(id,response.clone());
    }

    pub(super) fn make_attempt(&self, request: &Rc<MiniRequest>) -> (MiniRequestAttempt,CommanderStream<BackendResponse>) {
        let id = self.next_id();
        let request = MiniRequestAttempt::new(id,request);
        let stream = request.response().clone();
        self.add_pending(id,&stream);
        (request,stream)
    }

    pub(super) fn retrieve_attempt_by_response(&self, response: &BackendResponseAttempt) -> Option<CommanderStream<BackendResponse>> {
        let mut pending = lock!(self.pending);
        pending.remove(&response.message_id())
    }
}
