use peregrine_data::{JumpReq, JumpRes, JumpLocation};
use peregrine_toolkit::error::Error;
use crate::callbacks::Callbacks;

#[derive(Clone)]
pub(crate) struct Backend {
    callbacks: Callbacks
}

impl Backend {
    pub(crate) fn new(callbacks: Callbacks) -> Backend {
        Backend { callbacks }
    }

    pub(crate) fn jump(&self, req: &JumpReq) -> Result<JumpRes,Error> {
        let location = req.location();
        if let Some((stick,left,right)) = self.callbacks.jump(location)? {
            let location = JumpLocation { stick, left, right };
            Ok(JumpRes::Found(location))
        } else {
            Ok(JumpRes::NotFound)
        }
    }
}
