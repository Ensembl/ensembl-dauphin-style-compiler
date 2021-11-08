use peregrine_toolkit::cbor::{cbor_into_map, cbor_map_key};
use crate::{Assets, ProgramName, core::channel::Channel};
use serde_cbor::Value as CborValue;

pub struct BootstrapCommandResponse {
    program_name: ProgramName,
    channel_hi: Channel,
    channel_lo: Channel,
    assets: Assets,
}

impl BootstrapCommandResponse {
    pub fn decode(value: CborValue) -> Result<BootstrapCommandResponse,String> {
        let mut map = cbor_into_map(value)?;
        Ok(BootstrapCommandResponse {
            program_name: ProgramName::decode(cbor_map_key(&mut map,"boot")?)?,
            channel_hi: Channel::decode(cbor_map_key(&mut map,"hi")?)?,
            channel_lo: Channel::decode(cbor_map_key(&mut map,"lo")?)?,
            assets: Assets::decode(cbor_map_key(&mut map,"assets")?)?
        })
    }

    pub(crate) fn program_name(&self) -> &ProgramName { &self.program_name }
    pub(crate) fn assets(&self) -> &Assets { &self.assets }
    pub(crate) fn channel_hi(&self) -> &Channel { &self.channel_hi }
    pub(crate) fn channel_lo(&self) -> &Channel { &self.channel_lo }
}
