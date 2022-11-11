use std::{sync::Arc};

pub enum ReceivedDataType { Bytes, Booleans, Numbers, Strings }

// XXX merge with geometry builder as pattern in toolkit
#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub enum ReceivedData {
    Bytes(Arc<Vec<u8>>),
    Booleans(Arc<Vec<bool>>),
    Numbers(Arc<Vec<f64>>),
    Strings(Arc<Vec<String>>)
}

impl ReceivedData {
    pub fn new_arc_bytes(data: &Arc<Vec<u8>>) -> ReceivedData { 
        ReceivedData::Bytes(data.clone())
    }

    pub fn new_arc_booleans(data: &Arc<Vec<bool>>) -> ReceivedData { 
        ReceivedData::Booleans(data.clone())
    }

    pub fn new_arc_numbers(data: &Arc<Vec<f64>>) -> ReceivedData { 
        ReceivedData::Numbers(data.clone())
    }

    pub fn new_arc_strings(data: &Arc<Vec<String>>) -> ReceivedData { 
        ReceivedData::Strings(data.clone())
    }

    pub fn new_bytes(data: Vec<u8>) -> ReceivedData { Self::new_arc_bytes(&Arc::new(data)) }
    pub fn new_booleans(data: Vec<bool>) -> ReceivedData { Self::new_arc_booleans(&Arc::new(data)) }
    pub fn new_numbers(data: Vec<f64>) -> ReceivedData { Self::new_arc_numbers(&Arc::new(data)) }
    pub fn new_strings(data: Vec<String>) -> ReceivedData { Self::new_arc_strings(&Arc::new(data)) }

    pub fn variety(&self) -> ReceivedDataType {
        match self {
            ReceivedData::Bytes(_) => ReceivedDataType::Bytes,
            ReceivedData::Booleans(_) => ReceivedDataType::Booleans,
            ReceivedData::Numbers(_) => ReceivedDataType::Numbers,
            ReceivedData::Strings(_) => ReceivedDataType::Strings
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ReceivedData::Bytes(x) => x.len(),
            ReceivedData::Booleans(x) => x.len(),
            ReceivedData::Numbers(x) => x.len(),
            ReceivedData::Strings(x) => x.len(),
        }
    }

    pub fn data_as_bytes(&self) -> Result<&Arc<Vec<u8>>,()> {
        match self {
            ReceivedData::Bytes(x) => Ok(&x),
            _ => Err(())
        }
    }

    pub fn data_as_booleans(&self) -> Result<&Arc<Vec<bool>>,()> {
        match self {
            ReceivedData::Booleans(x) => Ok(&x),
            _ => Err(())
        }
    }

    pub fn data_as_numbers(&self) -> Result<&Arc<Vec<f64>>,()> {
        match self {
            ReceivedData::Numbers(x) => Ok(&x),
            _ => Err(())
        }
    }

    pub fn data_as_strings(&self) -> Result<&Arc<Vec<String>>,()> {
        match self {
            ReceivedData::Strings(x) => Ok(&x),
            _ => Err(())
        }
    }
}
