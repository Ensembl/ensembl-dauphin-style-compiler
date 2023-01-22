use peregrine_data::{JumpReq, JumpRes, JumpLocation, BootChannelReq, BootChannelRes, Assets, BackendNamespace, StickReq, StickRes, ExpandRes, ExpandReq, ProgramReq, ProgramRes, DataRequest, DataRes, SmallValuesRes, SmallValuesReq};
use peregrine_toolkit::error::Error;
use crate::{callbacks::Callbacks, sidecars::JsSidecar};

pub(crate) enum CallbackError {
    Internal(Error),
    External(String)
}

impl CallbackError {
    pub(crate) fn to_error(self) -> Error {
        match self {
            CallbackError::Internal(e) => e,
            CallbackError::External(_) => Error::fatal("unexpected callback error outside callback")
        }
    }
}

#[derive(Clone)]
pub(crate) struct Backend {
    backend_namespace: BackendNamespace,
    callbacks: Callbacks
}

impl Backend {
    pub(crate) fn new(backend_namespace: BackendNamespace, callbacks: Callbacks) -> Backend {
        Backend { backend_namespace, callbacks }
    }

    pub(crate) async fn jump(&self, req: &JumpReq) -> Result<(JumpRes,JsSidecar),CallbackError> {
        let location = req.location();
        let (result,sidecar) = self.callbacks.jump(location).await?;
        if let Some((stick,left,right)) = result {
            let location = JumpLocation { stick, left, right };
            Ok((JumpRes::Found(location),sidecar))
        } else {
            Ok((JumpRes::NotFound,sidecar))
        }
    }

    pub(crate) async fn boot(&self, _req: &BootChannelReq) -> Result<(BootChannelRes,JsSidecar),CallbackError> {
        let sidecar = self.callbacks.boot().await?;
        Ok((BootChannelRes::new(self.backend_namespace.clone(),Assets::empty(),Assets::empty(),Some(vec![15])),sidecar))
    }

    pub(crate) async fn data(&self, req: &DataRequest) -> Result<(DataRes,JsSidecar),CallbackError> {
        self.callbacks.data(req).await
    }

    pub(crate) async fn small_values(&self, req: &SmallValuesReq) -> Result<(SmallValuesRes,JsSidecar),CallbackError> {
        let out = self.callbacks.small_values(req.namespace(),req.column()).await?;
        Ok((SmallValuesRes::new(out.0),out.1))
    }

    pub(crate) async fn expansion(&self, req: &ExpandReq) -> Result<(ExpandRes,JsSidecar),CallbackError> {
        let sidecar = self.callbacks.expansion(req.name(),req.step()).await?;
        Ok((ExpandRes,sidecar))
    }

    pub(crate) async fn program(&self, req: &ProgramReq) -> Result<(ProgramRes,JsSidecar),CallbackError> {
        let name = req.name();
        let sidecar = self.callbacks.program(name.group(),name.name(),name.version()).await?;
        Ok((ProgramRes,sidecar))
    }

    pub(crate) async fn stickinfo(&self, req: &StickReq) -> Result<(StickRes,JsSidecar),CallbackError> {
        match self.callbacks.stickinfo(&req.id()).await? {
            (Some(stick),sidecar) => Ok((StickRes::Stick(stick),sidecar)),
            (None,sidecar) => Ok((StickRes::Unknown(req.id().get_id().to_string()),sidecar))
        }
    }
}
