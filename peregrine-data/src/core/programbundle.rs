use std::{collections::HashMap};
use peregrine_toolkit::{cbor::{cbor_as_str, cbor_into_drained_map, cbor_into_vec, check_array_len}, decompose_vec};
use serde_cbor::Value as CborValue;

pub struct SuppliedBundle {
    bundle_name: String,
    program: CborValue,
    names: HashMap<String,String> // in-channel name -> in-bundle name
}

impl SuppliedBundle {
    pub(crate) fn bundle_name(&self) -> &str { &self.bundle_name }
    pub(crate) fn program(&self) -> &CborValue { &self.program }
    pub(crate) fn name_map(&self) -> impl Iterator<Item=(&str,&str)> {
        self.names.iter().map(|(x,y)| (x as &str,y as &str))
    }

    pub fn decode(value: CborValue) -> Result<SuppliedBundle,String> {
        let mut seq = cbor_into_vec(value)?;
        check_array_len(&seq,3)?;
        decompose_vec!(seq,bundle_name,program,names);
        let names = cbor_into_drained_map(names)?.drain(..)
            .map::<Result<_,String>,_>(|(k,v)| Ok((k,cbor_as_str(&v)?.to_string())))
            .collect::<Result<HashMap<_,_>,_>>()?;
        Ok(SuppliedBundle {
            bundle_name: cbor_as_str(&bundle_name)?.to_string(),
            program,
           names
        })      
    }
}
