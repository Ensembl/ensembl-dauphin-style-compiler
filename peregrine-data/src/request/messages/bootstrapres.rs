use peregrine_toolkit::cbor::{cbor_as_number, cbor_into_map, cbor_into_vec, cbor_map_key, cbor_map_optional_key};
use crate::{Assets, ProgramName, core::channel::Channel};
use serde_cbor::Value as CborValue;

pub struct BootRes {
    program_name: ProgramName,
    channel_lo: Option<Channel>,
    assets: Assets,
    supports: Option<Vec<u32>>
}

fn decode_supports(value: CborValue) -> Result<Vec<u32>,String> {
    cbor_into_vec(value)?.drain(..).map(|x| cbor_as_number(&x)).collect::<Result<_,_>>()
}

impl BootRes {
    pub fn decode(value: CborValue) -> Result<BootRes,String> {
        let mut map = cbor_into_map(value)?;
        let supports = cbor_map_optional_key(&mut map,"supports")
            .map(|value| { decode_supports(value) })
            .transpose()?;
        Ok(BootRes {
            program_name: ProgramName::decode(cbor_map_key(&mut map,"boot")?)?,
            channel_lo: cbor_map_optional_key(&mut map,"lp").map(|x| Channel::decode(x)).transpose()?,
            assets: Assets::decode(cbor_map_key(&mut map,"assets")?)?,
            supports
        })
    }

    pub(crate) fn program_name(&self) -> &ProgramName { &self.program_name }
    pub(crate) fn assets(&self) -> &Assets { &self.assets }
    pub(crate) fn channel_lo(&self) -> Option<&Channel> { self.channel_lo.as_ref() }
    pub(crate) fn supports(&self) -> Option<&[u32]> { self.supports.as_ref().map(|x| &x[..]).clone() }
}
