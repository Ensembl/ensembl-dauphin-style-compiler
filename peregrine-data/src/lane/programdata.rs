use anyhow::{ anyhow as err };
use crate::request::data::DataResponse;
use peregrine_toolkit::lock;
use std::sync::{ Arc, Mutex };

pub struct ProgramDataData {
    data: Vec<Arc<DataResponse>>
}

fn munge(key: u32) -> u32 { key ^ 0xC3A58 }

impl ProgramDataData {
    fn new() -> ProgramDataData {
        ProgramDataData {
            data: vec![]
        }
    }

    fn add(&mut self, entry: Arc<DataResponse>) -> u32 {
        let out = munge(self.data.len() as u32);
        self.data.push(entry);
        out
    }

    fn get(&self, id: u32) -> anyhow::Result<Arc<DataResponse>> {
        Ok(self.data.get(munge(id) as usize).ok_or(err!("bad data id"))?.clone())
    }
}

#[derive(Clone)]
pub struct ProgramData(Arc<Mutex<ProgramDataData>>);

impl ProgramData {
    pub fn new() -> ProgramData {
        ProgramData(Arc::new(Mutex::new(ProgramDataData::new())))
    }

    pub fn get(&self, id: u32) -> anyhow::Result<Arc<DataResponse>> {
        Ok(lock!(self.0).get(id)?.clone())
    }

    pub fn add(&self, item: Arc<DataResponse>) -> u32 {
        lock!(self.0).add(item)
    }
}
