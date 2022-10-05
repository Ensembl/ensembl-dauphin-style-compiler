use peregrine_toolkit::cbor::{cbor_as_number, cbor_into_map, cbor_into_vec, cbor_map_key, cbor_map_optional_key};
use crate::{Assets, ProgramName, BackendNamespace};
use serde_cbor::Value as CborValue;

pub struct BootChannelRes {
    program_name: ProgramName,
    namespace: BackendNamespace,
    channel_assets: Assets,
    chrome_assets: Assets,
    supports: Option<Vec<u32>>
}

fn decode_supports(value: CborValue) -> Result<Vec<u32>,String> {
    cbor_into_vec(value)?.drain(..).map(|x| cbor_as_number(&x)).collect::<Result<_,_>>()
}

impl BootChannelRes {
    pub fn decode(value: CborValue) -> Result<BootChannelRes,String> {
        let mut map = cbor_into_map(value)?;
        let supports = cbor_map_optional_key(&mut map,"supports")
            .map(|value| { decode_supports(value) })
            .transpose()?;
        let channel = BackendNamespace::decode(cbor_map_key(&mut map, "namespace")?)?;
        let chrome_assets =
            cbor_map_optional_key(&mut map,"chrome-assets")
                .map(|value| Assets::decode(None,value)).transpose()?
                .unwrap_or_else(|| Assets::empty());
        Ok(BootChannelRes {
            program_name: ProgramName::decode(cbor_map_key(&mut map,"boot")?)?,
            namespace: channel.clone(),
            channel_assets: Assets::decode(Some(&channel),cbor_map_key(&mut map,"assets")?)?,
            chrome_assets,
            supports
        })
    }

    pub(crate) fn program_name(&self) -> &ProgramName { &self.program_name }
    pub(crate) fn channel_assets(&self) -> &Assets { &self.channel_assets }
    pub(crate) fn chrome_assets(&self) -> &Assets { &self.chrome_assets }
    pub(crate) fn namespace(&self) -> &BackendNamespace { &self.namespace }
    pub(crate) fn supports(&self) -> Option<&[u32]> { self.supports.as_ref().map(|x| &x[..]).clone() }
}
