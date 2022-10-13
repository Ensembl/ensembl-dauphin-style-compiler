use peregrine_data::{JumpReq, JumpRes, JumpLocation, BootChannelReq, BootChannelRes, Assets, BackendNamespace, StickReq, StickRes};
use peregrine_toolkit::error::Error;
use crate::callbacks::Callbacks;

#[derive(Clone)]
pub(crate) struct Backend {
    backend_namespace: BackendNamespace,
    callbacks: Callbacks
}

impl Backend {
    pub(crate) fn new(backend_namespace: BackendNamespace, callbacks: Callbacks) -> Backend {
        Backend { backend_namespace, callbacks }
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

    pub(crate) fn boot(&self, _req: &BootChannelReq) -> Result<BootChannelRes,Error> {
        self.callbacks.boot()?;
        Ok(BootChannelRes::new(None,self.backend_namespace.clone(),Assets::empty(),Assets::empty(),Some(vec![15])))
    }

    pub(crate) fn stickinfo(&self, req: &StickReq) -> Result<StickRes,Error> {
        match self.callbacks.stickinfo(&req.id())? {
            Some(stick) => Ok(StickRes::Stick(stick)),
            None => Ok(StickRes::Unknown(req.id().get_id().to_string()))
        }
    }
}
