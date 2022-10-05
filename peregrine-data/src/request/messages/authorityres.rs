use peregrine_toolkit::{cbor::{cbor_as_str, cbor_into_vec, check_array_len}, decompose_vec };
use crate::{index::stickauthority::Authority, BackendNamespace};
use serde_cbor::Value as CborValue;

pub struct AuthorityRes {
    channel: BackendNamespace,
    startup_name: String,
    lookup_name: String,
    jump_name: String
}

impl AuthorityRes {
    pub fn build(&self) -> Authority {
        Authority::new(&self.channel,&self.startup_name,&self.lookup_name,&self.jump_name)
    }

    pub fn decode(value: CborValue) -> Result<AuthorityRes,String> {
        let mut value = cbor_into_vec(value)?;
        check_array_len(&value,4)?;
        decompose_vec!(value,channel,startup,lookup,jump);
        Ok(AuthorityRes {
            channel: BackendNamespace::decode(channel)?,
            startup_name: cbor_as_str(&startup)?.to_string(),
            lookup_name: cbor_as_str(&lookup)?.to_string(),
            jump_name: cbor_as_str(&jump)?.to_string(),
        })
    }
}
