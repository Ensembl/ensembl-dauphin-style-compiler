use serde_derive::Deserialize;
use crate::{Assets, ProgramName, core::channel::Channel};

#[derive(Deserialize)]
pub struct BootstrapCommandResponse {
    #[serde(alias = "boot")]
    program_name: ProgramName,
    #[serde(alias = "hi")]
    channel_hi: Channel,
    #[serde(alias = "lo")]
    channel_lo: Channel,
    #[serde(default = "Assets::empty")]
    assets: Assets,
}

impl BootstrapCommandResponse {
    pub(crate) fn program_name(&self) -> &ProgramName { &self.program_name }
    pub(crate) fn assets(&self) -> &Assets { &self.assets }
    pub(crate) fn channel_hi(&self) -> &Channel { &self.channel_hi }
    pub(crate) fn channel_lo(&self) -> &Channel { &self.channel_lo }
}
