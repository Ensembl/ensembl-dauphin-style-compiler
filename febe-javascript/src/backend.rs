use peregrine_data::{JumpReq, JumpRes, JumpLocation, BootChannelReq, BootChannelRes, Assets, BackendNamespace, StickReq, StickRes, ExpandRes, ExpandReq};
use peregrine_toolkit::error::Error;
use crate::{callbacks::Callbacks, sidecars::JsSidecar};

#[derive(Clone)]
pub(crate) struct Backend {
    backend_namespace: BackendNamespace,
    callbacks: Callbacks
}

impl Backend {
    pub(crate) fn new(backend_namespace: BackendNamespace, callbacks: Callbacks) -> Backend {
        Backend { backend_namespace, callbacks }
    }

    pub(crate) async fn jump(&self, req: &JumpReq) -> Result<(JumpRes,JsSidecar),Error> {
        let location = req.location();
        let (result,sidecar) = self.callbacks.jump(location).await?;
        if let Some((stick,left,right)) = result {
            let location = JumpLocation { stick, left, right };
            Ok((JumpRes::Found(location),sidecar))
        } else {
            Ok((JumpRes::NotFound,sidecar))
        }
    }

    pub(crate) async fn boot(&self, _req: &BootChannelReq) -> Result<(BootChannelRes,JsSidecar),Error> {
        let sidecar = self.callbacks.boot().await?;
        Ok((BootChannelRes::new(self.backend_namespace.clone(),Assets::empty(),Assets::empty(),Some(vec![15])),sidecar))
    }

    pub(crate) async fn expansion(&self, req: &ExpandReq) -> Result<(ExpandRes,JsSidecar),Error> {
        let sidecar = self.callbacks.expansion(req.name(),req.step()).await?;
        Ok((ExpandRes,sidecar))
    }

    pub(crate) async fn stickinfo(&self, req: &StickReq) -> Result<(StickRes,JsSidecar),Error> {
        match self.callbacks.stickinfo(&req.id()).await? {
            (Some(stick),sidecar) => Ok((StickRes::Stick(stick),sidecar)),
            (None,sidecar) => Ok((StickRes::Unknown(req.id().get_id().to_string()),sidecar))
        }
    }
}
